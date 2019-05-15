use crate::sim::player::{PlayerKey, PlayerControls, FromMouse};
use crate::sim::chunk::SubIndex;
use super::*;
use super::chunk::ChunkState;
use glutin::dpi::LogicalPosition;

impl InputImpl {
    /// Process input events
    pub fn process_events(&mut self) {
        let mut events_loop = glutin::EventsLoop::new();
        ::std::mem::swap(&mut events_loop, &mut self.input_state.events_loop);
        events_loop.poll_events(|event| {
            use glutin::*;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => self.running = false,
                    WindowEvent::KeyboardInput {
                        input: glutin::KeyboardInput {
                            virtual_keycode: Some(glutin::VirtualKeyCode::Escape), ..
                        }, ..
                    } => self.running = false,
                    WindowEvent::Resized(logical_size) => {
                        let (w, h) = logical_size.into();
                        // TODO: Don't hardcode DPI and track HiDpiFactorChanged
                        self.input_state.window.resize(logical_size.to_physical(1.0));
                        // Update framebuffer sizes
                        gfx_window_glutin::update_views(&self.input_state.window, &mut self.rendering_state.data.out_color, &mut self.rendering_state.data.out_depth);
                        self.input_state.camera.resize_window(w, h);
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
                        self.input_state.keyboard_state.update_key(scancode, pressed);
                    },
                    WindowEvent::Focused(foc) => {
                        self.input_state.focused = foc;
                        if foc {
                            self.input_state.keyboard_state.clear();
                        }
                    },
                    WindowEvent::MouseInput { button, state, .. } => {
                        if button == MouseButton::Right && state == ElementState::Pressed {
                            println!("Player position: {:?}", self.input_state.camera.get_pos());
                            let player_chunk : ChunkPos = self.input_state.camera.get_pos().high();
                            let c = self.game_state.chunks.get(&player_chunk).unwrap().borrow();
                            println!("Player chunk: {:?} (fragment_count: {}, adj_chunks: {}, state: {:?})", player_chunk, c.fragments, c.adj_chunks, c.state);
                        } else {
                            self.input_state.mouse_state = state
                        }
                    },
                    _ => {},
                },
                Event::DeviceEvent { event, .. } => match event {
                    // TODO: Ensure this event is only received if the window is focused
                    DeviceEvent::Motion { axis, value } => {
                        match axis {
                            0 => self.input_state.camera.update_cursor(value, 0.0),
                            1 => self.input_state.camera.update_cursor(0.0, value),
                            _ => panic!("Unknown axis. Expected 0 or 1, found {}.", axis),
                        }
                    },
                    _ => {},
                },
                _ => {},
            }
        });
        ::std::mem::swap(&mut events_loop, &mut self.input_state.events_loop);
    }

    /// Process messages from other threads and queue chunk-related messages
    pub fn process_messages(&mut self) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                ToInput::NewChunkBuffer(pos, vertices) => {
                    assert!(vertices.len() % 3 == 0); // Triangles should have 3 vertices
                    //println!("Input: received vertex buffer @ {:?}", pos);
                    if let Some(ref chunk) = self.game_state.chunks.get_mut(&pos) {
                        chunk.borrow_mut().state = ChunkState::Meshed(
                            self.rendering_state
                                .factory
                                .create_vertex_buffer_with_slice(&vertices, ()),
                        );
                    }
                }
                ToInput::SetPos(pos) => {
                    self.input_state.camera.set_pos(pos.0);
                }
                message @ ToInput::NewChunkFragment(..) | message @ ToInput::NewChunkInfo(..) => {
                    self.pending_messages.push_back(message);
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

    /// Move camera and send keyboard state to the server
    pub fn move_camera(&mut self) {
        self.input_state.timer = Instant::now();
        let &mut InputImpl {
            ref network_tx,
            ref mut ticker,
            ref input_state,
            ..
        } = self;

        // Send keys
        if ticker.try_tick() {
            let keys = {
                let ks = &input_state.keyboard_state;
                let mut mask = PlayerControls::mouse(input_state.mouse_state);
                if ks.is_key_pressed(MOVE_FORWARD) {
                    mask |= PlayerKey::Forward
                }
                if ks.is_key_pressed(MOVE_LEFT) {
                    mask |= PlayerKey::Left
                }
                if ks.is_key_pressed(MOVE_BACKWARD) {
                    mask |= PlayerKey::Backward
                }
                if ks.is_key_pressed(MOVE_RIGHT) {
                    mask |= PlayerKey::Right
                }

                if ks.is_key_pressed(MOVE_UP) {
                    mask |= PlayerKey::Up
                }
                if ks.is_key_pressed(MOVE_DOWN) {
                    mask |= PlayerKey::Down
                }
                if ks.is_key_pressed(CONTROL) {
                    mask |= PlayerKey::Control
                }
                mask
            };
            let yp = input_state.camera.get_yaw_pitch();
            network_tx
                .send(ToNetwork::SetInput(PlayerInput {
                    keys,
                    yaw: yp[0],
                    pitch: yp[1],
                }))
                .unwrap();
        }
    }

    pub fn center_cursor(&mut self) {
        // Only move cursor if the window is focused
        // TODO: (bug?) When the window is opened the first time, but not focused, it is annoying
        // that the cursor constantly gets recentered. Also, might want to ignore mouse motion events fired
        // while the window was being loaded
        if self.input_state.focused {
            let (w, h): (f64, f64) = self.input_state.window.get_inner_size().unwrap().into();
            self.input_state
                .window
                .set_cursor_position(LogicalPosition::new(w / 2.0, h / 2.0))
                .unwrap();
        }
    }
}
