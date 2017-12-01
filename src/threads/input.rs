extern crate gfx;
extern crate glutin;
extern crate gfx_window_glutin;
extern crate image;
extern crate cgmath;

use std;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::collections::HashMap;
use std::path::Path;

use gfx::traits::FactoryExt;
use gfx::{Device, Factory};
use gfx::texture::{SamplerInfo, FilterMethod, WrapMode};
use self::glutin::{GlContext, MouseCursor};

use ::{CHUNK_SIZE, ColorFormat, DepthFormat, pipe, Vertex, Transform};
use ::core::messages::client::{ToInput, ToMeshing, ToNetwork};
use ::texture::{load_textures};
use ::block::{BlockRegistry, Chunk, ChunkPos, create_block_air, create_block_cube};
use ::render::{camera, frames};
use ::config;

use threads::utility::key_from_u64;

const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

pub fn start() {
    // Load config
    std::fs::create_dir_all(Path::new("cfg")).unwrap();
    let config = config::load_config(Path::new("cfg/cfg.toml"));

    // Window creation
    let mut events_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new()
        .with_title("Triangle example".to_string());
    let context = glutin::ContextBuilder::new()
        .with_vsync(false)
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)));
    let (window, mut device, mut factory, main_color, main_depth) = 
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

    let mut br = BlockRegistry::new();
    br.add_block(Box::new(air));
    br.add_block(Box::new(dirt));
    br.add_block(Box::new(grass));
    br.add_block(Box::new(wood));
    br.add_block(Box::new(leaves));

    let br = Arc::new(br);
    let mut chunks = HashMap::new();

    // TODO: Completely useless, this is just used to fill the PSO
    let chunk = Chunk::new();
    let cube: Vec<Vertex> = chunk.calculate_mesh(&br);

    // Channels
    let rx;
    let meshing_tx;
    let network_tx;
    // Start threads
    {
        // Input
        let (input_t, input_r) = channel();
        // Meshing
        let (meshing_t, meshing_r) = channel();
        // Network
        let (network_t, network_r) = channel();

        {
            let input_tx = input_t.clone();
            let br2 = br.clone();
            thread::spawn(move || {
                ::threads::meshing::start(meshing_r, input_tx, br2);
            });
            println!("Started meshing thread");
        }

        {
            let meshing_tx = meshing_t.clone();
            thread::spawn(move || {
                ::threads::network::start(network_r, meshing_tx);
            });
            println!("Started network thread");
        }

        rx = input_r;
        meshing_tx = meshing_t;
        network_tx= network_t;
    }

    // Render data
    let (vertex_buffer, _) = factory.create_vertex_buffer_with_slice(&cube, ());
    let transform_buffer = factory.create_constant_buffer(1);
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        transform: transform_buffer,
        //image: (load_texture(&mut factory, "assets/grass_side.png"), sampler),
        //image: (load_textures(&mut factory).0, sampler),
        image: (atlas, sampler),
        out_color: main_color,
        out_depth: main_depth,
    };
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    // TODO: Frame buffer size and window size might be different
    let (w, h) = window.get_inner_size().unwrap();
    let mut cam = camera::Camera::new(w, h, &config);
    let mut keys = ::input::KeyboardState::new();

    window.set_cursor(MouseCursor::Crosshair);
    let mut focused = false;

    // Main loop
    let mut running = true;
    let mut timer = ::std::time::SystemTime::now();
    let mut fc = frames::FrameCounter::new();
    let mut cnt: u32 = 0;
    while running {
        // Event handling
        events_loop.poll_events(|event| {
            use self::glutin::*;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Closed => running = false,
                    WindowEvent::KeyboardInput {
                        input: glutin::KeyboardInput {
                            virtual_keycode: Some(glutin::VirtualKeyCode::Escape), ..
                        }, ..
                    } => running = false,
                    WindowEvent::Resized(w, h) => {
                        window.resize(w, h);
                        gfx_window_glutin::update_views(&window, &mut data.out_color, &mut data.out_depth);
                        cam.resize_window(w, h);
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
                        keys.update_key(key_from_u64(scancode as u64).unwrap(), pressed);
                    },
                    WindowEvent::Focused(foc) => {
                        focused = foc;
                        if foc {
                            keys.clear();
                        }
                    },
                    _ => {},
                },
                Event::DeviceEvent { event, .. } => match event {
                    // TODO: Ensure this event is only received if the window is focused
                    DeviceEvent::Motion { axis, value } => {
                        match axis {
                            0 => cam.update_cursor(value as f32, 0.0),
                            1 => cam.update_cursor(0.0, value as f32),
                            _ => panic!("Unknown axis. Expected 0 or 1, found {}.", axis),
                        }
                    },
                    _ => {},
                },
                _ => {},
            }
        });

        // Message handling
        while let Ok(message) = rx.try_recv() {
            match message {
                ToInput::NewChunkBuffer(pos, vertices) => {
                    assert!(vertices.len()%3 == 0); // Triangles should have 3 vertices
                    if let Some(buffer @ &mut None) = chunks.get_mut(&pos) {
                        *buffer = Some(factory.create_vertex_buffer_with_slice(&vertices, ()));
                    }
                },
            }
        }

        // Frames
        let frames = fc.frame();
        cnt += 1;
        cnt %= 200;
        if cnt == 0 {
            println!("FPS: {}", frames);
        }

        // Ticking
        let elapsed = timer.elapsed().unwrap();
        cam.tick(elapsed.subsec_nanos() as f32/1_000_000_000.0 +  elapsed.as_secs() as f32, &keys);
        timer = ::std::time::SystemTime::now();
        // Only move cursor if the window is focused
        // TODO: (bug?) When the window is opened the first time, but not focused, it is annoying
        // that the cursor constantly gets recentered. Also, might want to ignore mouse motion events fired
        // while the window was being loaded
        if focused {
            let (w, h) = window.get_inner_size().unwrap();
            window.set_cursor_position((w/2) as i32, (h/2) as i32).unwrap();
        }

        let player_chunk = cam.get_pos();
        let player_chunk = ChunkPos(
            player_chunk.0 as i64 / CHUNK_SIZE as i64,
            player_chunk.1 as i64 / CHUNK_SIZE as i64,
            player_chunk.2 as i64 / CHUNK_SIZE as i64);

        // Rendering
        let render_dist = config.render_distance;
        for i in -render_dist..(render_dist+1) {
            for j in -render_dist..(render_dist+1) {
                for k in -render_dist..(render_dist+1) {
                    let pck = &player_chunk;
                    let pos = ChunkPos(pck.0 + i, pck.1 + j, pck.2 + k);
                    chunks.entry(pos.clone()).or_insert_with(|| {
                        println!("Input: asked for buffer @ {:?}", pos);
                        meshing_tx.send(ToMeshing::AllowChunk(pos.clone())).unwrap();
                        network_tx.send(ToNetwork::NewChunk(pos)).unwrap();
                        None
                    });
                }
            }
        }
        

        chunks.retain(|pos, _| {
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

        let mut transform = Transform {
            view_proj: cam.get_view_projection().into(),
            model: [[0.0; 4]; 4],
        };
        
        encoder.clear(&data.out_color, CLEAR_COLOR);
        encoder.clear_depth(&data.out_depth, 1.0);

        for (pos, buffer) in &mut chunks {
            match buffer {
                &mut Some(ref mut buff) => {
                    transform.model = cgmath::Matrix4::from_translation((CHUNK_SIZE as f32) * cgmath::Vector3::new(pos.0 as f32, pos.1 as f32, pos.2 as f32)).into();
                    encoder.update_buffer(&data.transform,
                        &[transform],
                        0).unwrap();
                    // Evil swap hack
                    std::mem::swap(&mut data.vbuf, &mut buff.0);
                    encoder.draw(&buff.1, &pso, &data);
                    std::mem::swap(&mut data.vbuf, &mut buff.0);
                }
                &mut None => (),
            }
        }
        encoder.flush(&mut device);

        window.swap_buffers().unwrap();
        device.cleanup();
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