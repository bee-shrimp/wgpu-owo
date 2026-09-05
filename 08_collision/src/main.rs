#![deny(clippy::all)]
#![forbid(unsafe_code)]

// ---------------------------------------------------------------- imports

use color_eyre::Result;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
use app::App;

mod config;
mod ecs;
mod renderer;
mod sound;
mod sprite;
mod world;

mod animation;

// ---------------------------------------------------------------- main

fn main() -> Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    if let Some(err) = app.take_error() {
        // eprintln!("{err:#}");
        log::error!("{err}");
    }

    Ok(())
}
