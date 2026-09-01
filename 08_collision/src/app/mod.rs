// ---------------------------------------------------------------- App struct
use anyhow::{Context, Error, Result};

use std::sync::Arc;
use std::time::Instant;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::renderer::Renderer;
use crate::sound::SoundPlayer;
use crate::world::World;

pub mod input;
use input::InputHandler;

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
            self.fail(event_loop, err);
        }
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
                self.input.resize(size);
            }

            // ---------------------------------------------------- redraw
            //
            WindowEvent::RedrawRequested => match renderer.render() {
                Ok(()) => {}
                Err(err) => {
                    self.fail(event_loop, err);
                }
            },

            // ---------------------------------------------------- detect cursor position
            //
            WindowEvent::CursorMoved { position, .. } => {
                self.input.update_cursor_pos(position);
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
            self.fail(event_loop, err);
        }
    }
}

impl App {
    fn fail(&mut self, event_loop: &ActiveEventLoop, err: Error) {
        self.error = Some(err);
        event_loop.exit();
    }

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

        self.input = InputHandler::new(window_size);

        // -------------------------------------------------------- create renderer/sound player

        let renderer = smol::block_on(Renderer::new(window, event_loop, instances));
        let sound_player = smol::block_on(SoundPlayer::new());

        self.renderer = Some(renderer.context("failed to create renderer")?);
        self.sound = Some(sound_player.context("failed to create sound player")?);

        // -------------------------------------------------------- init renderer

        self.renderer
            .as_mut()
            .context("failed to find renderer")?
            .update(instances)?;

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

        let Some(sound) = self.sound.as_mut() else {
            return Err(anyhow::anyhow!("failed to find sound player"));
        };

        // -------------------------------------------------------- update world

        self.world
            .update(&input_state, sound, dt)
            .context("failed to update world")?;
        let instances = self.world.update_instances();

        // -------------------------------------------------------- update renderer

        let Some(renderer) = self.renderer.as_mut() else {
            return Err(anyhow::anyhow!("failed to find renderer"));
        };

        renderer
            .update(instances)
            .context("failed to update renderer")?;
        renderer.get_window().request_redraw();

        Ok(())
    }
}
