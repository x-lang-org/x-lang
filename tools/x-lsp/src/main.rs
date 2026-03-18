//! X Language LSP Server entry point

use anyhow::Result;
use log::info;

mod server;
mod handlers;
mod analysis;
mod state;
mod utils;

fn main() -> Result<()> {
    env_logger::init();
    info!("Starting X Language LSP Server");

    let mut server = server::LspServer::new()?;
    server.run()?;

    Ok(())
}
