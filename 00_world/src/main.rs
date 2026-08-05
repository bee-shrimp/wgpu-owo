#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

mod renderer;
use renderer::Renderer;

mod world;
use world::{Direction, Rect};

const TARGET_FPS: f64 = 60.0;
const FRAME_TIME: f64 = 1.0 / TARGET_FPS;
const EPSILON: f32 = 0.001;

#[derive(Default)]
struct App {
    renderer: Option<Renderer>,
    pressed_keys: HashSet<KeyCode>,
    rect: Option<Rect>,
    last_frame_time: Option<Instant>,
    next_frame_time: Option<Instant>,
    last_draw_data: (f32, f32),
    need_redraw: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let time = Instant::now();
        self.last_frame_time = Some(time);
        self.next_frame_time = Some(time + Duration::from_secs_f64(FRAME_TIME));
        self.last_draw_data = (0.0, 0.0);
        self.need_redraw = true;
        // --------------------------------------------------------------------------- create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        // --------------------------------------------------------------------------- create renderer
        self.renderer = Some(pollster::block_on(Renderer::new(window)).unwrap());

        // --------------------------------------------------------------------------- init other fields
        self.rect = Some(Rect::new());
    }

    // ------------------------------------------------------------------------------- handle window events
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let renderer = match &mut self.renderer {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            // ----------------------------------------------------------------------- close window
            WindowEvent::CloseRequested => {
                println!("close requested; stopping");
                event_loop.exit();
            }

            // ----------------------------------------------------------------------- resize
            WindowEvent::Resized(size) => renderer.resize(size.width, size.height),

            // ----------------------------------------------------------------------- redraw
            WindowEvent::RedrawRequested => {
                if self.need_redraw {
                    match renderer.render() {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("{e}");
                            event_loop.exit();
                        }
                    }
                }
            }

            // ----------------------------------------------------------------------- handle key inputs
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => match state {
                ElementState::Pressed => {
                    self.pressed_keys.insert(code);
                }
                ElementState::Released => {
                    self.pressed_keys.remove(&code);
                }
            },

            _ => (),
        }
    }

    // ------------------------------------------------------------------------------- things to do after everyting else
    #[allow(unused_variables)]
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let last = self.last_frame_time.unwrap();
        let mut next = self.next_frame_time.unwrap();

        let dt = now.duration_since(last).as_secs_f32();

        let gap = 1.0 / dt; //fps
        // println!("{:.2}", gap);

        next += Duration::from_secs_f64(FRAME_TIME);
        if now < next {
            std::thread::sleep(next - now);
        } else {
            self.next_frame_time = Some(now)
        }
        self.last_frame_time = Some(now);

        let keys = &self.pressed_keys;
        if keys.is_empty() {
            self.need_redraw = false
        } else {
            let renderer = self.renderer.as_mut().unwrap();
            let rect = self.rect.as_mut().unwrap();
            let mut direction_x: Direction = Direction::Still;
            let mut direction_y: Direction = Direction::Still;

            if keys.contains(&KeyCode::ArrowUp) {
                direction_y = Direction::Up
            }

            if keys.contains(&KeyCode::ArrowLeft) {
                direction_x = Direction::Left
            }
            if keys.contains(&KeyCode::ArrowDown) {
                direction_y = Direction::Down
            }

            if keys.contains(&KeyCode::ArrowRight) {
                direction_x = Direction::Right
            }

            let direction = (direction_x, direction_y);
            let rect_pos = rect.update(dt, direction);

            let x_dist = self.last_draw_data.0 - rect_pos.0;
            let y_dist = self.last_draw_data.1 - rect_pos.1;

            if x_dist.abs() >= EPSILON || y_dist.abs() >= EPSILON {
                self.need_redraw = true;
                self.last_draw_data = rect_pos;
                renderer.update(rect_pos);
                renderer.get_window().request_redraw()
            } else {
                self.need_redraw = false;
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();

    Ok(())
}
