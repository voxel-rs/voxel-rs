extern crate cobalt;
extern crate gfx;
extern crate gfx_device_gl;
extern crate glutin;
extern crate gfx_window_glutin;
extern crate image;
extern crate cgmath;
extern crate net2;

use std;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::Arc;
use std::thread;
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::time::Instant;
use std::cell::RefCell;

use gfx::traits::FactoryExt;
use gfx::{Device, Factory};
use gfx::texture::{SamplerInfo, FilterMethod, WrapMode};
use self::glutin::{GlContext, MouseCursor};
use self::net2::UdpSocketExt;

use ::{CHUNK_SIZE, ColorFormat, DepthFormat, pipe, PlayerData, Vertex, Transform};
use ::core::messages::client::{ToInput, ToMeshing, ToNetwork};
use ::texture::{load_textures};
use ::block::{BlockRegistry, Chunk, ChunkInfo, ChunkPos, ChunkSidesArray, create_block_air, create_block_cube};
use ::input::KeyboardState;
use ::render::frames::FrameCounter;
// TODO: Don't use "*"
use ::render::camera::*;
use ::config::{Config, load_config};
use ::texture::TextureRegistry;
use ::util::Ticker;
use ::player::PlayerInput;

type PipeDataType = pipe::Data<gfx_device_gl::Resources>;
type PsoType = gfx::PipelineState<gfx_device_gl::Resources, pipe::Meta>;
type EncoderType = gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>;

const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

const ADJ_CHUNKS: [[i64; 3]; 6] = [
    [ 0,  0, -1],
    [ 0,  0,  1],
    [ 1,  0,  0],
    [-1,  0,  0],
    [ 0,  1,  0],
    [ 0, -1,  0],
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

struct InputImpl {
    running: bool,
    config: Arc<Config>,
    rx: Receiver<ToInput>,
    /// Chunk updates that need the chunks to be loaded in memory first depending on the player's position
    pending_messages: VecDeque<ToInput>,
    meshing_tx: Sender<ToMeshing>,
    network_tx: Sender<ToNetwork>,
    game_state: ClientGameState,
    rendering_state: RenderingState,
    debug_info: DebugInfo,
    game_registries: GameRegistries,
    ticker: Ticker,
}

struct ClientGameState {
    pub window: glutin::GlWindow,
    pub focused: bool,
    pub events_loop: glutin::EventsLoop,
    pub keyboard_state: KeyboardState,
    pub camera: Camera,
    pub timer: Instant,
    pub chunks: HashMap<ChunkPos, RefCell<ChunkData>>,
}

struct RenderingState {
    pub device: gfx_device_gl::Device,
    pub factory: gfx_device_gl::Factory,
    pub pso: PsoType,
    pub data: PipeDataType,
    pub encoder: EncoderType,
}

struct GameRegistries {
    pub block_registry: Arc<BlockRegistry>,
    pub texture_registry: TextureRegistry,
}

struct DebugInfo {
    pub fc: FrameCounter,
    pub cnt: u32,
}

type BufferHandle3D = (gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>, gfx::Slice<gfx_device_gl::Resources>);

/// Chunk information stored by the client
struct ChunkData {
    /// The chunk data itself
    pub chunk: Chunk,
    /// How many fragments have been received
    pub fragments: usize,
    /// What adjacent chunks are loaded. This is a bit mask, and 1 means loaded.
    /// All chunks loaded means that adj_chunks == 0b00111111
    pub adj_chunks: u8,
    /// The loaded bits
    pub chunk_info: ChunkInfo,
    /// The chunk's state
    pub state: ChunkState,
}

/// A client chunk's state
enum ChunkState {
    Unmeshed,
    Meshing,
    Meshed(BufferHandle3D),
}


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
    pub fn new() -> Self {
        // Load config
        std::fs::create_dir_all(Path::new("cfg")).unwrap();
        let config = Arc::new(load_config(Path::new("cfg/cfg.toml")));

        // Window creation
        let events_loop = glutin::EventsLoop::new();
        let builder = glutin::WindowBuilder::new()
            .with_title("voxel-rs".to_string());
        let context = glutin::ContextBuilder::new()
            .with_vsync(false)
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)));
        let (window, device, mut factory, main_color, main_depth) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, context, &events_loop);

        let shader_set = factory.create_shader_set(
            include_bytes!("../shader/vertex_150.glslv"),
            include_bytes!("../shader/vertex_150.glslf")
        ).unwrap();

        let pso = factory.create_pipeline_state(
            &shader_set,
            self::gfx::Primitive::TriangleList,
            self::gfx::state::Rasterizer::new_fill().with_cull_back(),
            pipe::new()
        ).unwrap();

        // Sampler
        let sampler = factory.create_sampler(SamplerInfo::new(FilterMethod::Scale, WrapMode::Clamp));

        // Blocks
        let (atlas, texture_registry) = load_textures(&mut factory);
        let air = create_block_air();
        let dirt = create_block_cube(["dirt"; 6], &texture_registry);
        let grass = create_block_cube(["grass_side", "grass_side", "grass_side", "grass_side", "grass_top", "dirt"], &texture_registry);
        let wood = create_block_cube(["wood_side", "wood_side", "wood_side", "wood_side", "wood_top", "wood_top"], &texture_registry);
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
            use self::cobalt::{BinaryRateLimiter, Client, Config, NoopPacketModifier, Server, UdpSocket};
            // Input
            let (input_t, input_r) = channel();
            // Meshing
            let (meshing_t, meshing_r) = channel();
            // Network
            let (network_t, network_r) = channel();
            // Client-server
            let cfg = Config {
                send_rate: config.tick_rate,
                packet_max_size: 576, // 576 is the IPv4 "minimum reassembly buffer size"
                connection_init_threshold: ::std::time::Duration::new(1, 0),
                connection_drop_threshold: ::std::time::Duration::new(4, 0),
                ..Config::default()
            };
            let mut server = Server::<UdpSocket, BinaryRateLimiter, NoopPacketModifier>::new(cfg);
            let mut client = Client::<UdpSocket, BinaryRateLimiter, NoopPacketModifier>::new(cfg);

            {
                let input_tx = input_t.clone();
                let br2 = br.clone();
                thread::spawn(move || {
                    ::client::meshing::start(meshing_r, input_tx, br2);
                });
                println!("Started meshing thread");
            }

            {
                let input_tx = input_t.clone();
                thread::spawn(move || {
                    thread::sleep(std::time::Duration::from_millis(2000));
                    client.connect("127.0.0.1:1106").expect("Failed to bind to socket.");
                    client.socket().unwrap().as_raw_udp_socket().set_recv_buffer_size(1024*1024*8).unwrap();
                    client.socket().unwrap().as_raw_udp_socket().set_send_buffer_size(1024*1024*8).unwrap();
                    ::client::network::start(network_r, input_tx, client);
                    //client.disconnect();
                });
                println!("Started network thread");
            }

            {
                let (game_tx, game_rx) = channel();
                let (network_tx, network_rx) = channel();
                let (worldgen_tx, worldgen_rx) = channel();
                let game_t = game_tx.clone();
                thread::spawn(move || {
                    match server.listen("0.0.0.0:1106") {
                        Ok(()) =>  {
                            server.socket().unwrap().as_raw_udp_socket().set_recv_buffer_size(1024*1024*8).unwrap();
                            server.socket().unwrap().as_raw_udp_socket().set_send_buffer_size(1024*1024*8).unwrap();
                            ::server::network::start(network_rx, game_t, server);
                            //server.shutdown();
                        },
                        Err(e) => {
                            println!("Failed to bind to socket. Error : {:?}", e);
                        }
                    }
                    //server.shutdown();
                });
                let config = config.clone();
                thread::spawn(move || {
                    ::server::game::start(game_rx, network_tx, worldgen_tx, config);
                });

                thread::spawn(move || {
                    ::server::worldgen::start(worldgen_rx, game_tx);
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
        let (w, h) = window.get_inner_size().unwrap();
        let cam = Camera::new(w, h, &config);

        window.set_cursor(MouseCursor::Crosshair);

        // Send render distance
        network_tx.send(ToNetwork::SetRenderDistance(config.render_distance as u64)).unwrap();

        // Create object
        Self {
            running: true,
            config,
            rx,
            pending_messages: VecDeque::new(),
            meshing_tx,
            network_tx,
            game_state: ClientGameState {
                window,
                focused: false,
                events_loop,
                keyboard_state: KeyboardState::new(),
                camera: cam,
                timer: Instant::now(),
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

    pub fn keep_running(&self) -> bool {
        self.running
    }

    pub fn process_events(&mut self) {
        let mut events_loop = glutin::EventsLoop::new();
        ::std::mem::swap(&mut events_loop, &mut self.game_state.events_loop);
        events_loop.poll_events(|event| {
            use self::glutin::*;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Closed => self.running = false,
                    WindowEvent::KeyboardInput {
                        input: glutin::KeyboardInput {
                            virtual_keycode: Some(glutin::VirtualKeyCode::Escape), ..
                        }, ..
                    } => self.running = false,
                    WindowEvent::Resized(w, h) => {
                        self.game_state.window.resize(w, h);
                        // Update framebuffer sizes
                        gfx_window_glutin::update_views(&self.game_state.window, &mut self.rendering_state.data.out_color, &mut self.rendering_state.data.out_depth);
                        self.game_state.camera.resize_window(w, h);
                    },
                    WindowEvent::KeyboardInput {
                        input: glutin::KeyboardInput {
                            scancode, state, ..
                        }, ..
                    } => {
                        let pressed: bool = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                        //println!("Key {} pressed ? {}", scancode, pressed);
                        self.game_state.keyboard_state.update_key(scancode, pressed);
                    },
                    WindowEvent::Focused(foc) => {
                        self.game_state.focused = foc;
                        if foc {
                            self.game_state.keyboard_state.clear();
                        }
                    },
                    WindowEvent::MouseInput { button, state, .. } => {
                        if button == MouseButton::Left && state == ElementState::Pressed {
                            println!("Player position: {:?}", self.game_state.camera.get_pos());
                            let player_chunk = self.game_state.camera.get_pos().chunk_pos();
                            let c = self.game_state.chunks.get(&player_chunk).unwrap().borrow();
                            println!("Player chunk: {:?} (fragment_count: {}, adj_chunks: {}, state: {:?})", player_chunk, c.fragments, c.adj_chunks, c.state);
                        }
                    },
                    _ => {},
                },
                Event::DeviceEvent { event, .. } => match event {
                    // TODO: Ensure this event is only received if the window is focused
                    DeviceEvent::Motion { axis, value } => {
                        match axis {
                            0 => self.game_state.camera.update_cursor(value, 0.0),
                            1 => self.game_state.camera.update_cursor(0.0, value),
                            _ => panic!("Unknown axis. Expected 0 or 1, found {}.", axis),
                        }
                    },
                    _ => {},
                },
                _ => {},
            }
        });
        ::std::mem::swap(&mut events_loop, &mut self.game_state.events_loop);
    }

    pub fn process_messages(&mut self) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                ToInput::NewChunkBuffer(pos, vertices) => {
                    assert!(vertices.len()%3 == 0); // Triangles should have 3 vertices
                    //println!("Input: received vertex buffer @ {:?}", pos);
                    if let Some(ref chunk) = self.game_state.chunks.get_mut(&pos) {
                        chunk.borrow_mut().state = ChunkState::Meshed(self.rendering_state.factory.create_vertex_buffer_with_slice(&vertices, ()));
                    }
                },
                ToInput::SetPos(pos) => {
                    self.game_state.camera.set_pos(pos.0);
                },
                message @ ToInput::NewChunkFragment(..) | message @ ToInput::NewChunkInfo(..) => {
                    self.pending_messages.push_back(message);
                },
            }
        }
    }

    pub fn process_chunk_messages(&mut self) {
        for message in self.pending_messages.drain(..) {
            match message {
                ToInput::NewChunkFragment(pos, fpos, frag) => {
                    if let Some(data) = self.game_state.chunks.get(&pos) {
                        let mut data = &mut *data.borrow_mut();
                        let index = fpos.0[0]*32 + fpos.0[1];
                        // New fragment
                        if data.chunk_info[index/32]&(1 << (index%32)) == 0 {
                            data.chunk_info[index/32] |= 1 << (index%32);
                            data.chunk.blocks[fpos.0[0]][fpos.0[1]] = *frag;
                            data.fragments += 1;
                            // TODO: check that the chunk is in render_distance but NOT in (render_distance; rander_distance+1]
                            // Update adjacent chunks too
                            Self::check_finalize_chunk(pos, data, &self.game_state.chunks, &self.game_registries.block_registry);
                        }
                    }
                },
                ToInput::NewChunkInfo(pos, info) => {
                    if let Some(data) = self.game_state.chunks.get(&pos) {
                        let mut data = &mut *data.borrow_mut();
                        for (from, to) in info.iter().zip(data.chunk_info.iter_mut()) {
                            data.fragments -= to.count_ones() as usize;
                            *to |= *from;
                            data.fragments += to.count_ones() as usize;
                        }
                        // Update adjacent chunks
                        Self::check_finalize_chunk(pos, data, &self.game_state.chunks, &self.game_registries.block_registry);
                    }
                },
                _ => unreachable!(),
            }
        }
    }

    fn check_finalize_chunk(pos: ChunkPos, data: &mut ChunkData, chunks: &HashMap<ChunkPos, RefCell<ChunkData>>, br: &BlockRegistry) {
        if data.fragments == CHUNK_SIZE * CHUNK_SIZE {
            for face in 0..6 {
                let adj = ADJ_CHUNKS[face];
                let mut pos = pos;
                for i in 0..3 {
                    pos.0[i] += adj[i];
                }
                if let Some(c) = chunks.get(&pos) {
                    let mut adj_chunk = c.borrow_mut();
                    if adj_chunk.adj_chunks&(1 << face) == 0 {
                        adj_chunk.adj_chunks |= 1 << face;
                        // We update that adjacent chunk's sides with the current chunk !
                        Self::update_side(
                            // It should be the opposite face from the adjacent chunk's POV, so we XOR 1 to flip the last bit
                            face^1,
                            &data.chunk,
                            &mut adj_chunk.chunk.sides,
                            br
                        );
                    }
                }
                else {
                    println!("Warning: LOST INFORMATION. Chunk {:?} is not loaded!", pos);
                }
            }
        }
    }

    pub fn update_frame_count(&mut self) {
        let frames = self.debug_info.fc.frame();
        self.debug_info.cnt += 1;
        self.debug_info.cnt %= 200;
        if self.debug_info.cnt == 0 {
            println!("FPS: {}", frames);
        }
    }

    pub fn move_camera(&mut self) {
        self.game_state.timer = Instant::now();
        let &mut InputImpl {
            ref network_tx,
            ref mut ticker,
            ref game_state,
            ..
        } = self;

        // Send updated position to network
        if ticker.try_tick() {
            let keys = {
                let ks = &game_state.keyboard_state;
                let mut mask = 0;
                mask |= ((ks.is_key_pressed(MOVE_FORWARD) as u8) << 0) as u8;
                mask |= ((ks.is_key_pressed(MOVE_LEFT) as u8) << 1) as u8;
                mask |= ((ks.is_key_pressed(MOVE_BACKWARD) as u8) << 2) as u8;
                mask |= ((ks.is_key_pressed(MOVE_RIGHT) as u8) << 3) as u8;
                mask |= ((ks.is_key_pressed(MOVE_UP) as u8) << 4) as u8;
                mask |= ((ks.is_key_pressed(MOVE_DOWN) as u8) << 5) as u8;
                mask |= ((ks.is_key_pressed(CONTROL) as u8) << 6) as u8;
                mask
            };
            let yp = game_state.camera.get_yaw_pitch();
            network_tx.send(ToNetwork::SetInput(PlayerInput { keys, yaw: yp[0], pitch: yp[1] })).unwrap();
        }
    }

    pub fn center_cursor(&mut self) {
        // Only move cursor if the window is focused
        // TODO: (bug?) When the window is opened the first time, but not focused, it is annoying
        // that the cursor constantly gets recentered. Also, might want to ignore mouse motion events fired
        // while the window was being loaded
        if self.game_state.focused {
            let (w, h) = self.game_state.window.get_inner_size().unwrap();
            self.game_state.window.set_cursor_position((w/2) as i32, (h/2) as i32).unwrap();
        }
    }

    pub fn fetch_close_chunks(&mut self) {
        let player_chunk = self.game_state.camera.get_pos().chunk_pos();

        // Fetch new close chunks
        // render_distance+2 because we need to store information about the adjacent chunks
        // even if they are not yet loaded. We use +2 instead of +1 to have a small margin,
        // just in case the packets don't arrive in the right order.
        let render_dist = self.config.render_distance+2;
        for i in -render_dist..(render_dist+1) {
            for j in -render_dist..(render_dist+1) {
                for k in -render_dist..(render_dist+1) {
                    let mut pos = ChunkPos([i, j, k]);
                    for x in 0..3 {
                        pos.0[x] += player_chunk.0[x];
                    }
                    self.game_state.chunks.entry(pos.clone()).or_insert_with(|| {
                        RefCell::new(ChunkData {
                            chunk: Chunk::new(),
                            fragments: 0,
                            adj_chunks: 0,
                            chunk_info: [0; CHUNK_SIZE * CHUNK_SIZE / 32],
                            state: ChunkState::Unmeshed,
                        })
                    });
                }
            }
        }

        // Trash far chunks
        let render_dist = (self.config.render_distance+2) as u64;
        self.game_state.chunks.retain(|pos, _| {
            pos.orthogonal_dist(player_chunk) <= render_dist
        });

        // Start meshing for new chunks
        for (pos, chunk) in self.game_state.chunks.iter() {
            let mut c = chunk.borrow_mut();
            // TODO: FRAGMENT_COUNT const
            if c.adj_chunks == 0b00111111 && c.fragments == CHUNK_SIZE * CHUNK_SIZE {
                let mut update_state = false;
                if let ChunkState::Unmeshed = c.state {
                    update_state = true;
                    self.meshing_tx.send(ToMeshing::ComputeChunkMesh(*pos, c.chunk.clone())).unwrap();
                }
                if update_state {
                    c.state = ChunkState::Meshing;
                }
            }
        }
    }

    /// Reversed means the internal faces of the chunk
    fn get_range(x: i64, reversed: bool) -> std::ops::Range<usize> {
        match x {
            0 => 0..CHUNK_SIZE,
            1 => if reversed { (CHUNK_SIZE-1)..CHUNK_SIZE } else { 0..1 },
            -1 => if reversed { 0..1 } else { (CHUNK_SIZE-1)..CHUNK_SIZE },
            _ => panic!("Impossible value"),
        }
    }

    fn update_side(face: usize, c: &Chunk, sides: &mut ChunkSidesArray, br: &BlockRegistry) {
        let adj = ADJ_CHUNKS[face];
        for (int_x, ext_x) in Self::get_range(adj[0], true).zip(Self::get_range(adj[0], false)) {
            for (int_y, ext_y) in Self::get_range(adj[1], true).zip(Self::get_range(adj[1], false)) {
                for (int_z, ext_z) in Self::get_range(adj[2], true).zip(Self::get_range(adj[2], false)) {
                    if !br.get_block(c.blocks[ext_x][ext_y][ext_z]).is_opaque() {
                        sides[int_x][int_y][int_z] |= 1 << face;
                    }
                }
            }
        }
    }

    pub fn render(&mut self) {
        let state = &mut self.rendering_state;

        // The transform buffer
        let mut transform = Transform {
            view_proj: self.game_state.camera.get_view_projection().cast::<f32>().into(),
            model: [[0.0; 4]; 4],
        };

        // The player data buffer
        let player_data = PlayerData {
            direction: self.game_state.camera.get_cam_dir().cast::<f32>().into(),
        };

        state.encoder.update_buffer(&state.data.player_data, &[player_data], 0).unwrap();

        state.encoder.clear(&state.data.out_color, CLEAR_COLOR);
        state.encoder.clear_depth(&state.data.out_depth, 1.0);

        // Render every chunk independently
        for (pos, chunk) in self.game_state.chunks.iter_mut() {
            if let ChunkState::Meshed(ref mut buff) = chunk.borrow_mut().state {
                transform.model = cgmath::Matrix4::from_translation(
                    (CHUNK_SIZE as f32)
                    * cgmath::Vector3::new(pos.0[0] as f32, pos.0[1] as f32, pos.0[2] as f32)
                ).into();
                state.encoder.update_buffer(&state.data.transform,
                    &[transform],
                    0).unwrap();
                // Evil swap hack
                std::mem::swap(&mut state.data.vbuf, &mut buff.0);
                state.encoder.draw(&buff.1, &state.pso, &state.data);
                std::mem::swap(&mut state.data.vbuf, &mut buff.0);
            }
        }
        state.encoder.flush(&mut state.device);

        self.game_state.window.swap_buffers().unwrap();
        state.device.cleanup();
    }
}
