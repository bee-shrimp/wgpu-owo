#![deny(clippy::all)]
#![forbid(unsafe_code)]

// ---------------------------------------------------------------- imports

use anyhow::Context;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
use app::App;

mod config;
mod ecs;
mod renderer;
mod sprite;
mod world;

// ---------------------------------------------------------------- main

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new().context("failed to create event loop")?;

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).context("failed to run app")?;

    Ok(())
}
