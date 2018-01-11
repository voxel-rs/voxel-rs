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
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use gfx::traits::FactoryExt;
use gfx::{Device, Factory};
use gfx::texture::{SamplerInfo, FilterMethod, WrapMode};
use self::glutin::{GlContext, MouseCursor};
use self::net2::UdpSocketExt;

use ::{CHUNK_SIZE, ColorFormat, DepthFormat, pipe, PlayerData, Vertex, Transform};
use ::core::messages::client::{ToInput, ToMeshing, ToNetwork};
use ::texture::{load_textures};
use ::block::{BlockRegistry, Chunk, ChunkPos, create_block_air, create_block_cube};
use ::input::KeyboardState;
use ::render::frames::FrameCounter;
use ::render::camera::Camera;
use ::config::{Config, load_config};
use ::texture::TextureRegistry;
use ::util::Ticker;

type PipeDataType = pipe::Data<gfx_device_gl::Resources>;
type PsoType = gfx::PipelineState<gfx_device_gl::Resources, pipe::Meta>;
type EncoderType = gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>;

const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

pub fn start() {
    let mut implementation = InputImpl::new();

    while implementation.keep_running() {
        // Event handling
        implementation.process_events();

        // Message handling
        implementation.process_messages();

        // Frames
        implementation.update_frame_count();

        // Ticking
        implementation.move_camera();

        // Center cursor
        // TODO: Draw custom crossbar instead of using the system cursor.
        implementation.center_cursor();

        // Fetch new chunks, and trash far chunks.
        implementation.fetch_close_chunks();

        // Render scene
        implementation.render();
    }
}

struct InputImpl {
    running: bool,
    config: Config,
    rx: Receiver<ToInput>,
    meshing_tx: Sender<ToMeshing>,
    network_tx: Sender<ToNetwork>,
    game_state: ClientGameState,
    rendering_state: RenderingState,
    debug_info: DebugInfo,
    #[allow(dead_code)] game_registries: GameRegistries,
    ticker: Ticker,
}

struct ClientGameState {
    pub window: glutin::GlWindow,
    pub focused: bool,
    pub events_loop: glutin::EventsLoop,
    pub keyboard_state: KeyboardState,
    pub camera: Camera,
    pub timer: Instant,
}

struct RenderingState {
    pub device: gfx_device_gl::Device,
    pub factory: gfx_device_gl::Factory,
    pub pso: PsoType,
    pub data: PipeDataType,
    pub encoder: EncoderType,
    pub chunks: HashMap<ChunkPos, Option<(gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>, gfx::Slice<gfx_device_gl::Resources>)>>,
}

struct GameRegistries {
    pub block_registry: Arc<BlockRegistry>,
    pub texture_registry: TextureRegistry,
}

struct DebugInfo {
    pub fc: FrameCounter,
    pub cnt: u32,
}

impl InputImpl {
    pub fn new() -> Self {
        // Load config
        std::fs::create_dir_all(Path::new("cfg")).unwrap();
        let config = load_config(Path::new("cfg/cfg.toml"));

        // Window creation
        let events_loop = glutin::EventsLoop::new();
        let builder = glutin::WindowBuilder::new()
            .with_title("voxel-rs".to_string());
        let context = glutin::ContextBuilder::new()
            .with_vsync(false)
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)));
        let (window, device, mut factory, main_color, main_depth) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, context, &events_loop);

        let pso = factory.create_pipeline_simple(
            include_bytes!("../shader/vertex_150.glslv"),
            include_bytes!("../shader/vertex_150.glslf"),
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

        let mut br = BlockRegistry::new();
        br.add_block(Box::new(air));
        br.add_block(Box::new(dirt));
        br.add_block(Box::new(grass));
        br.add_block(Box::new(wood));
        br.add_block(Box::new(leaves));
        br.add_block(Box::new(stone));

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
                send_rate: 2500, // TODO: This is not suitable for normal connections.
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
                let meshing_tx = meshing_t.clone();
                thread::spawn(move || {
                    thread::sleep(std::time::Duration::from_millis(2000));
                    client.connect("127.0.0.1:1106").expect("Failed to bind to socket.");
                    client.socket().unwrap().as_raw_udp_socket().set_recv_buffer_size(1024*1024*8).unwrap();
                    client.socket().unwrap().as_raw_udp_socket().set_send_buffer_size(1024*1024*8).unwrap();
                    ::client::network::start(network_r, meshing_tx, client);
                    //client.disconnect();
                });
                println!("Started network thread");
            }

            {
                let (game_tx, game_rx) = channel();
                let (network_tx, network_rx) = channel();
                thread::spawn(move || {
                    match server.listen("0.0.0.0:1106") {
                        Ok(()) =>  {
                            server.socket().unwrap().as_raw_udp_socket().set_recv_buffer_size(1024*1024*8).unwrap();
                            server.socket().unwrap().as_raw_udp_socket().set_send_buffer_size(1024*1024*8).unwrap();
                            ::server::network::start(network_rx, game_tx, server);
                            //server.shutdown();
                        },
                        Err(e) => {
                            println!("Failed to bind to socket. Error : {:?}", e);
                        }
                    }
                    //server.shutdown();
                });

                thread::spawn(move || {
                    ::server::game::start(game_rx, network_tx);
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
            meshing_tx,
            network_tx,
            game_state: ClientGameState {
                window,
                focused: false,
                events_loop,
                keyboard_state: KeyboardState::new(),
                camera: cam,
                timer: Instant::now(),
            },
            rendering_state: RenderingState {
                device,
                factory,
                pso,
                data,
                encoder,
                chunks: HashMap::new(),
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
                        }
                    },
                    _ => {},
                },
                Event::DeviceEvent { event, .. } => match event {
                    // TODO: Ensure this event is only received if the window is focused
                    DeviceEvent::Motion { axis, value } => {
                        match axis {
                            0 => self.game_state.camera.update_cursor(value as f32, 0.0),
                            1 => self.game_state.camera.update_cursor(0.0, value as f32),
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
                    println!("Input: received vertex buffer @ {:?}", pos);
                    if let Some(buffer @ &mut None) = self.rendering_state.chunks.get_mut(&pos) {
                        *buffer = Some(self.rendering_state.factory.create_vertex_buffer_with_slice(&vertices, ()));
                    }
                },
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
        let elapsed = self.game_state.timer.elapsed();
        self.game_state.camera.tick(elapsed.subsec_nanos() as f32/1_000_000_000.0 +  elapsed.as_secs() as f32, &self.game_state.keyboard_state);
        self.game_state.timer = Instant::now();

        // Send updated position to network
        if self.ticker.try_tick() {
            self.network_tx.send(ToNetwork::SetPos({
                let p = self.game_state.camera.get_pos();
                (p.0 as f64, p.1 as f64, p.2 as f64)
            })).unwrap();
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
        let meshing_tx = self.meshing_tx.clone();

        let player_chunk = self.game_state.camera.get_pos();
        let player_chunk = ChunkPos(
            player_chunk.0 as i64 / CHUNK_SIZE as i64,
            player_chunk.1 as i64 / CHUNK_SIZE as i64,
            player_chunk.2 as i64 / CHUNK_SIZE as i64);

        // Fetch new close chunks
        let render_dist = self.config.render_distance;
        for i in -render_dist..(render_dist+1) {
            for j in -render_dist..(render_dist+1) {
                for k in -render_dist..(render_dist+1) {
                    let pck = &player_chunk;
                    let pos = ChunkPos(pck.0 + i, pck.1 + j, pck.2 + k);
                    self.rendering_state.chunks.entry(pos.clone()).or_insert_with(|| {
                        println!("Input: asked for buffer @ {:?}", pos);
                        meshing_tx.send(ToMeshing::AllowChunk(pos.clone())).unwrap();
                        None
                    });
                }
            }
        }

        // Trash far chunks
        self.rendering_state.chunks.retain(|pos, _| {
            if
                abs(pos.0 - player_chunk.0) > render_dist ||
                abs(pos.1 - player_chunk.1) > render_dist ||
                abs(pos.2 - player_chunk.2) > render_dist {

                meshing_tx.send(ToMeshing::RemoveChunk(pos.clone())).unwrap();
                false
            }
            else {
                true
            }
        });
    }

    pub fn render(&mut self) {
        let state = &mut self.rendering_state;

        // The transform buffer
        let mut transform = Transform {
            view_proj: self.game_state.camera.get_view_projection().into(),
            model: [[0.0; 4]; 4],
        };

        // The player data buffer
        let player_data = PlayerData {
            direction: self.game_state.camera.get_cam_dir(),
        };

        state.encoder.update_buffer(&state.data.player_data, &[player_data], 0).unwrap();

        state.encoder.clear(&state.data.out_color, CLEAR_COLOR);
        state.encoder.clear_depth(&state.data.out_depth, 1.0);

        // Render every chunk independently
        for (pos, buffer) in state.chunks.iter_mut() {
            match buffer {
                &mut Some(ref mut buff) => {
                    transform.model = cgmath::Matrix4::from_translation(
                        (CHUNK_SIZE as f32)
                        * cgmath::Vector3::new(pos.0 as f32, pos.1 as f32, pos.2 as f32)
                    ).into();
                    state.encoder.update_buffer(&state.data.transform,
                        &[transform],
                        0).unwrap();
                    // Evil swap hack
                    std::mem::swap(&mut state.data.vbuf, &mut buff.0);
                    state.encoder.draw(&buff.1, &state.pso, &state.data);
                    std::mem::swap(&mut state.data.vbuf, &mut buff.0);
                }
                &mut None => (),
            }
        }
        state.encoder.flush(&mut state.device);

        self.game_state.window.swap_buffers().unwrap();
        state.device.cleanup();
    }
}

fn abs(x: i64) -> i64 {
    if x < 0 {
        -x
    }
    else {
        x
    }
}
