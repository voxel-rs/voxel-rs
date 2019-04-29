//! The input thread is the main client-side thread.
//! It is responsible for everything but networking and meshing.
//! This module contains the code for starting the client, hereby starting the game.
//! The `game` submodule is reponsible for chunk handling, rendering and meshing.
//! The `input` submodule manages interactions between the player, this thread, and the other client side threads.

use glutin::ElementState;
use std;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use gfx::texture::{FilterMethod, SamplerInfo, WrapMode};
use gfx::traits::FactoryExt;
use gfx::Factory;
use glutin::MouseCursor;

use crate::block::{
    create_block_air, create_block_cube, BlockRegistry
};
use crate::sim::chunk::{
    ChunkPos
};
use self::chunk::{Chunk, ChunkData, ChunkState};
use crate::config::{load_config, Config};
use crate::core::messages::client::{ToInput, ToMeshing, ToNetwork};
use crate::input::KeyboardState;
use crate::sim::player::PlayerInput;
use crate::render::camera::*;
use crate::render::frames::FrameCounter;
use crate::texture::load_textures;
use crate::texture::TextureRegistry;
use crate::util::Ticker;
use crate::{pipe, ColorFormat, DepthFormat, PlayerData, Transform, Vertex, CHUNK_SIZE};

mod game;
mod input;
pub mod chunk;

type PipeDataType = pipe::Data<gfx_device_gl::Resources>;
type PsoType = gfx::PipelineState<gfx_device_gl::Resources, pipe::Meta>;
type EncoderType = gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>;

const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

const ADJ_CHUNKS: [[i64; 3]; 6] = [
    [0, 0, -1],
    [0, 0, 1],
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
];

pub fn start() {
    let mut implementation = InputImpl::new();

    while implementation.keep_running() {
        // Event handling
        implementation.process_events();

        // Center cursor
        // TODO: Draw custom crossbar instead of using the system cursor.
        implementation.center_cursor();

        // Message handling
        implementation.process_messages();

        // Ticking
        implementation.move_camera();

        // Fetch new chunks, send new ones to meshing and trash far chunks.
        implementation.fetch_close_chunks();

        // Process queued chunk messages
        implementation.process_chunk_messages();

        // Render scene
        implementation.render();

        // Frames
        implementation.update_frame_count();
    }
}

/// Client input thread's state
struct InputImpl {
    running: bool,
    config: Arc<Config>,
    rx: Receiver<ToInput>,
    /// Chunk updates that need the chunks to be loaded in memory first depending on the player's position
    pending_messages: VecDeque<ToInput>,
    meshing_tx: Sender<ToMeshing>,
    network_tx: Sender<ToNetwork>,
    input_state: InputState,
    game_state: ClientGameState,
    rendering_state: RenderingState,
    debug_info: DebugInfo,
    game_registries: GameRegistries,
    ticker: Ticker,
}

/// Input-related state
struct InputState {
    pub window: glutin::GlWindow,
    pub focused: bool,
    pub events_loop: glutin::EventsLoop,
    pub keyboard_state: KeyboardState,
    pub camera: Camera,
    pub timer: Instant,
    pub mouse_state : ElementState
}

/// Game-related state
struct ClientGameState {
    pub chunks: HashMap<ChunkPos, RefCell<ChunkData>>,
}

/// Rendering-related state
struct RenderingState {
    pub device: gfx_device_gl::Device,
    pub factory: gfx_device_gl::Factory,
    pub pso: PsoType,
    pub data: PipeDataType,
    pub encoder: EncoderType,
}

/// Registries
struct GameRegistries {
    pub block_registry: Arc<BlockRegistry>,
    pub texture_registry: TextureRegistry,
}

/// Debug information
struct DebugInfo {
    pub fc: FrameCounter,
    pub cnt: u32,
}

type BufferHandle3D = (
    gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>,
    gfx::Slice<gfx_device_gl::Resources>,
);

impl std::fmt::Debug for ChunkState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            &ChunkState::Unmeshed => write!(f, "ChunkState::Unmeshed"),
            &ChunkState::Meshing => write!(f, "ChunkState::Meshing"),
            &ChunkState::Meshed(_) => write!(f, "ChunkState::Meshed(_)"),
        }
    }
}

impl InputImpl {
    /// Start the client and the server (i.e. the whole game)
    pub fn new() -> Self {
        // Load config
        std::fs::create_dir_all(Path::new("cfg")).unwrap();
        let config = Arc::new(load_config(Path::new("cfg/cfg.toml")));

        // Window creation
        let events_loop = glutin::EventsLoop::new();
        let builder = glutin::WindowBuilder::new().with_title("voxel-rs".to_string());
        let context = glutin::ContextBuilder::new()
            .with_vsync(false)
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)));
        let (window, device, mut factory, main_color, main_depth) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, context, &events_loop)
                .unwrap();

        let shader_set = factory
            .create_shader_set(
                include_bytes!("../../shader/vertex_150.glslv"),
                include_bytes!("../../shader/vertex_150.glslf"),
            )
            .unwrap();

        let pso = factory
            .create_pipeline_state(
                &shader_set,
                gfx::Primitive::TriangleList,
                gfx::state::Rasterizer::new_fill().with_cull_back(),
                pipe::new(),
            )
            .unwrap();

        // Sampler
        let sampler =
            factory.create_sampler(SamplerInfo::new(FilterMethod::Scale, WrapMode::Clamp));

        // Blocks
        let (atlas, texture_registry) = load_textures(&mut factory);
        let air = create_block_air();
        let dirt = create_block_cube(["dirt"; 6], &texture_registry);
        let grass = create_block_cube(
            [
                "grass_side",
                "grass_side",
                "grass_side",
                "grass_side",
                "grass_top",
                "dirt",
            ],
            &texture_registry,
        );
        let wood = create_block_cube(
            [
                "wood_side",
                "wood_side",
                "wood_side",
                "wood_side",
                "wood_top",
                "wood_top",
            ],
            &texture_registry,
        );
        let leaves = create_block_cube(["leaves"; 6], &texture_registry);
        let stone = create_block_cube(["stone"; 6], &texture_registry);
        let coal = create_block_cube(["ore_coal"; 6], &texture_registry);

        let mut br = BlockRegistry::new();
        br.add_block(Box::new(air));
        br.add_block(Box::new(dirt));
        br.add_block(Box::new(grass));
        br.add_block(Box::new(wood));
        br.add_block(Box::new(leaves));
        br.add_block(Box::new(stone));
        br.add_block(Box::new(coal));

        let br = Arc::new(br);

        // Channels
        let rx;
        let meshing_tx;
        let network_tx;
        // Start threads
        {
            use crate::simple::network::{SimpleClient, SimpleServer};
            // Input
            let (input_t, input_r) = channel();
            // Meshing
            let (meshing_t, meshing_r) = channel();
            // Network
            let (network_t, network_r) = channel();
            // Client-server
            let (to_server, from_client) = channel();
            let (to_client, from_server) = channel();
            let client = SimpleClient::new(from_server, to_server);
            let server = SimpleServer::new(from_client, to_client);
            {
                let input_tx = input_t.clone();
                let br2 = br.clone();
                thread::spawn(move || {
                    crate::client::meshing::start(meshing_r, input_tx, br2);
                });
                println!("Started meshing thread");
            }

            {
                let input_tx = input_t.clone();
                thread::spawn(move || {
                    crate::client::network::start(network_r, input_tx, client);
                });
                println!("Started network thread");
            }

            {
                let (game_tx, game_rx) = channel();
                let (network_tx, network_rx) = channel();
                let (worldgen_tx, worldgen_rx) = channel();
                let game_t = game_tx.clone();
                thread::spawn(move || {
                    crate::server::network::start(network_rx, game_t, server);
                });
                let config = config.clone();
                thread::spawn(move || {
                    crate::server::game::start(game_rx, network_tx, worldgen_tx, config);
                });

                thread::spawn(move || {
                    crate::sim::worldgen::start(worldgen_rx, game_tx);
                });
            }

            rx = input_r;
            meshing_tx = meshing_t;
            network_tx = network_t;
        }

        // TODO: Completely useless, this is just used to fill the PSO
        let chunk = Chunk::new();
        let cube: Vec<Vertex> = chunk.calculate_mesh(&br);

        // Render data
        let (vertex_buffer, _) = factory.create_vertex_buffer_with_slice(&cube, ());
        let transform_buffer = factory.create_constant_buffer(1);
        let player_data_buffer = factory.create_constant_buffer(1);
        let data = pipe::Data {
            vbuf: vertex_buffer,
            transform: transform_buffer,
            player_data: player_data_buffer,
            //image: (load_texture(&mut factory, "assets/grass_side.png"), sampler),
            //image: (load_textures(&mut factory).0, sampler),
            image: (atlas, sampler),
            out_color: main_color,
            out_depth: main_depth,
        };
        let encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

        // TODO: Frame buffer size and window size might be different
        let (w, h) = window.get_inner_size().unwrap().into();
        let cam = Camera::new(w, h, &config);

        window.set_cursor(MouseCursor::Crosshair);

        // Send render distance
        network_tx
            .send(ToNetwork::SetRenderDistance(config.render_distance as u64))
            .unwrap();

        // Create object
        Self {
            running: true,
            config,
            rx,
            pending_messages: VecDeque::new(),
            meshing_tx,
            network_tx,
            input_state: InputState {
                window,
                focused: false,
                events_loop,
                keyboard_state: KeyboardState::new(),
                camera: cam,
                timer: Instant::now(),
                mouse_state: ElementState::Released
            },
            game_state: ClientGameState {
                chunks: HashMap::new(),
            },
            rendering_state: RenderingState {
                device,
                factory,
                pso,
                data,
                encoder,
            },
            debug_info: DebugInfo {
                fc: FrameCounter::new(),
                cnt: 0,
            },
            game_registries: GameRegistries {
                block_registry: br,
                texture_registry: texture_registry,
            },
            ticker: Ticker::from_tick_rate(30),
        }
    }

    /// Still running ?
    pub fn keep_running(&self) -> bool {
        self.running
    }
}
