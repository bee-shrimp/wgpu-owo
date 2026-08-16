#![deny(clippy::all)]
#![forbid(unsafe_code)]

// ------------------------------------------------------------------- imports

use std::sync::Arc;

use anyhow::Context;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

mod renderer;
use renderer::Renderer;

mod world;
use world::World;

mod ecs;
use crate::ecs::{Pos, Size};

mod input;
use input::InputHandler;

// ------------------------------------------------------------------- App struct

#[derive(Default)]
struct App {
    renderer: Option<Renderer>,
    world: World,
    input: InputHandler,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // ----------------------------------------------------------- create window object

        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .expect("failed to create window"),
        );

        // ----------------------------------------------------------- create world

        self.world = World::default();
        self.world.init().expect("failed to init world");

        let instances = self.world.get_instances();

        // ----------------------------------------------------------- init input handler

        let window_size = window.inner_size();

        self.input = InputHandler::new(Size {
            w: window_size.width as f32,
            h: window_size.height as f32,
        });

        // ----------------------------------------------------------- create renderer

        self.renderer = Some(
            pollster::block_on(Renderer::new(window, event_loop, instances))
                .expect("failed to create renderer"),
        );

        // ----------------------------------------------------------- init renderer

        self.renderer
            .as_mut()
            .expect("failed to find renderer")
            .update(instances);
    }

    // --------------------------------------------------------------- handle window events

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        match event {
            // ------------------------------------------------------- close window
            //
            WindowEvent::CloseRequested => {
                println!("close requested; stopping");
                event_loop.exit();
            }

            // ------------------------------------------------------- resize
            //
            WindowEvent::Resized(size) => {
                renderer.resize(size.width, size.height);
                self.input.resize(Size {
                    w: size.width as f32,
                    h: size.height as f32,
                });
            }

            // ------------------------------------------------------- redraw
            //
            WindowEvent::RedrawRequested => match renderer.render() {
                Ok(()) => {}
                Err(e) => {
                    log::error!("{e}");
                    event_loop.exit();
                }
            },

            // ------------------------------------------------------- detect cursor position
            //
            WindowEvent::CursorMoved { position, .. } => {
                self.input.update_cursor_pos(Pos {
                    x: position.x as f32,
                    y: position.y as f32,
                });
            }

            // ------------------------------------------------------- handle mouse inputs
            //
            WindowEvent::MouseInput { button, state, .. } => match state {
                ElementState::Pressed => self.input.button_pressed(button),
                ElementState::Released => self.input.button_released(button),
            },

            // ------------------------------------------------------- handle key inputs
            //
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => match state {
                ElementState::Pressed => self.input.insert_key(code),
                ElementState::Released => self.input.remove_key(code),
            },

            _ => (),
        }
    }

    // --------------------------------------------------------------- things to do after everyting else

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        //
        // ----------------------------------------------------------- quit if key q is pressed

        if self.input.has_key(KeyCode::KeyQ) {
            event_loop.exit();
        }

        // ----------------------------------------------------------- return if paused and space is not pressed

        if !self.world.is_running() && !self.input.has_key(KeyCode::Space) {
            return;
        }

        // ----------------------------------------------------------- pause/resume if space is pressed

        if self.input.has_key(KeyCode::Space) {
            self.world.toggle_running();
            return;
        }

        // ----------------------------------------------------------- mouse state update

        self.input.update_mouse_state();
        if !self.input.has_triggered(MouseButton::Left) {
            return;
        };

        // ----------------------------------------------------------- update world if clicked

        let click_pos = self
            .input
            .get_click_pos(MouseButton::Left)
            .expect("failed to get click pos");
        self.world
            .update(click_pos)
            .expect("failed to update world");

        // ----------------------------------------------------------- update

        let renderer = self.renderer.as_mut().expect("failed to find renderer");

        let instances = self.world.get_instances();

        renderer.update(instances);
        renderer.get_window().request_redraw();
    }
}

// ------------------------------------------------------------------- main

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new().context("failed to create event loop")?;

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).context("failed to run app")?;

    Ok(())
}
