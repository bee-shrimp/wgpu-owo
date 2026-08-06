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
use world::{Direction, Pos, World};

const TARGET_FPS: f64 = 60.0;
const FRAME_TIME: f64 = 1.0 / TARGET_FPS;
const EPSILON: f32 = 0.01;

#[derive(Default)]
struct App {
    renderer: Option<Renderer>,
    world: World,
    pressed_keys: HashSet<KeyCode>,
    last_frame_time: Option<Instant>,
    next_frame_time: Option<Instant>,
    last_draw_data: Pos,
    need_redraw: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // --------------------------------------------------------------------------- create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        // --------------------------------------------------------------------------- create renderer
        self.renderer = Some(pollster::block_on(Renderer::new(window)).unwrap());

        // --------------------------------------------------------------------------- init other fields
        self.world = World::new();

        let time = Instant::now();
        self.last_frame_time = Some(time);
        self.next_frame_time = Some(time + Duration::from_secs_f64(FRAME_TIME));
        self.last_draw_data = Pos::default();
        self.need_redraw = true;
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
        // --------------------------------------------------------------------------- time related
        let now = Instant::now();
        let dt: f32;
        if let Some(last) = self.last_frame_time {
            dt = now.duration_since(last).as_secs_f32()
        } else {
            dt = 0.1
        };
        // let gap = 1.0 / dt; //fps
        // println!("{:.2}", gap);

        let mut next = self.next_frame_time.unwrap();
        next += Duration::from_secs_f64(FRAME_TIME);
        if now < next {
            std::thread::sleep(next - now);
        } else {
            self.next_frame_time = Some(now)
        }
        self.last_frame_time = Some(now);

        // --------------------------------------------------------------------------- return if paused or no input
        let keys = &self.pressed_keys;

        if keys.is_empty() {
            self.need_redraw = false;
            return;
        }

        if !self.world.is_running() && !keys.contains(&KeyCode::Space) {
            return;
        }

        if keys.contains(&KeyCode::Space) {
            self.world = self.world.toggle_running();
            return;
        }

        if !self.world.is_running() {
            return;
        }

        // --------------------------------------------------------------------------- update
        let direction = keys_to_direction(keys);
        self.world = self.world.update(dt, direction);

        let rect_pos = self.world.get_rect_pos();

        if has_moved(&self.last_draw_data, &rect_pos) {
            self.need_redraw = true;
            self.last_draw_data = rect_pos;
            let renderer = self.renderer.as_mut().unwrap();
            renderer.update((rect_pos.x, rect_pos.y));
            renderer.get_window().request_redraw()
        } else {
            self.need_redraw = false;
        }
    }
}

fn keys_to_direction(keys: &HashSet<KeyCode>) -> (Direction, Direction) {
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

    (direction_x, direction_y)
}

fn has_moved(last_draw_data: &Pos, pos: &Pos) -> bool {
    let x_dist = last_draw_data.x - pos.x;
    let y_dist = last_draw_data.y - pos.y;

    x_dist.abs() >= EPSILON || y_dist.abs() >= EPSILON
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();

    Ok(())
}
