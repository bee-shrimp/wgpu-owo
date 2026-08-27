// ---------------------------------------------------------------- App struct
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Error, Result};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::renderer::Renderer;

use crate::world::World;

use crate::ecs::{Pos, Size};

pub mod input;
use input::InputHandler;

use crate::sound::SoundPlayer;

// ---------------------------------------------------------------- App struct

#[derive(Default)]
pub struct App {
    renderer: Option<Renderer>,
    sound: Option<SoundPlayer>,
    world: World,
    input: InputHandler,
    last_frame_time: Option<Instant>,
    error: Option<Error>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(err) = self.init(event_loop) {
            self.error = Some(err);
            event_loop.exit();
        };
    }

    // ------------------------------------------------------------ handle window events

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        match event {
            // ---------------------------------------------------- close window
            //
            WindowEvent::CloseRequested => {
                println!("close requested; stopping");
                event_loop.exit();
            }

            // ---------------------------------------------------- resize
            //
            WindowEvent::Resized(size) => {
                renderer.resize(size.width, size.height);
                self.input.resize(Size {
                    w: size.width as f32,
                    h: size.height as f32,
                });
            }

            // ---------------------------------------------------- redraw
            //
            WindowEvent::RedrawRequested => match renderer.render() {
                Ok(_) => {}
                Err(e) => {
                    log::error!("{e}");
                    event_loop.exit();
                }
            },

            // ---------------------------------------------------- detect cursor position
            //
            WindowEvent::CursorMoved { position, .. } => {
                self.input.update_cursor_pos(Pos {
                    x: position.x as f32,
                    y: position.y as f32,
                });
            }

            // ---------------------------------------------------- handle mouse inputs
            //
            WindowEvent::MouseInput { button, state, .. } => match state {
                ElementState::Pressed => self.input.button_pressed(button),
                ElementState::Released => self.input.button_released(button),
            },

            // ---------------------------------------------------- handle key inputs
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

    // ------------------------------------------------------------ things to do after everything else

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // -------------------------------------------------------- error handling
        if let Err(err) = self.update(event_loop) {
            self.error = Some(err);
            event_loop.exit();
        };
    }
}

impl App {
    pub fn take_error(&mut self) -> Option<Error> {
        self.error.take()
    }

    fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        // -------------------------------------------------------- create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .context("failed to create window")?,
        );

        // -------------------------------------------------------- create world

        self.world = World::default();
        self.world.init().context("failed to init world")?;

        let instances = self.world.update_instances();

        // -------------------------------------------------------- init input handler

        let window_size = window.inner_size();

        self.input = InputHandler::new(Size {
            w: window_size.width as f32,
            h: window_size.height as f32,
        });

        // -------------------------------------------------------- create renderer/sound player

        let (renderer, sound_player) = smol::block_on(smol::future::zip(
            Renderer::new(window, event_loop, instances),
            SoundPlayer::new(),
        ));

        self.renderer = Some(renderer.context("failed to create renderer")?);
        self.sound = Some(sound_player.context("failed to create sound player")?);

        // -------------------------------------------------------- init renderer

        self.renderer
            .as_mut()
            .context("failed to find renderer")?
            .update(instances);

        // -------------------------------------------------------- init other fields

        self.last_frame_time = Some(Instant::now());
        self.error = None;

        Ok(())
    }

    fn update(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        // -------------------------------------------------------- calculate dt

        let now = Instant::now();

        // 0.016 = 60fps
        let dt = self.last_frame_time.map_or(0.016, |last| {
            now.duration_since(last).as_secs_f32().min(0.1)
        });

        self.last_frame_time = Some(now);

        // -------------------------------------------------------- quit if key q is pressed

        if self.input.has_key(KeyCode::KeyQ) {
            event_loop.exit();
        }

        // -------------------------------------------------------- return if paused and space is not pressed

        if !self.world.is_running() && !self.input.has_key(KeyCode::Space) {
            return Ok(());
        }

        // -------------------------------------------------------- pause/resume if space is pressed

        if self.input.has_key(KeyCode::Space) {
            self.world.toggle_running();
            return Ok(());
        }

        // -------------------------------------------------------- get input state

        let input_state = self.input.get_input_state();

        self.input.update_for_next_frame();

        // -------------------------------------------------------- sound for world

        let Some(sound) = self.sound.as_ref() else {
            return Err(anyhow::anyhow!("failed to find sound player"));
        };

        // -------------------------------------------------------- update world

        self.world.update(&input_state, sound, dt)?;

        // -------------------------------------------------------- update renderer

        let Some(renderer) = self.renderer.as_mut() else {
            return Err(anyhow::anyhow!("failed to find renderer"));
        };

        let instances = self.world.update_instances();

        renderer.update(instances);
        renderer.get_window().request_redraw();

        Ok(())
    }
}
