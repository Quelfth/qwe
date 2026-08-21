use std::io;

use crate::{
    aprintln::aprintln,
    lsp::client::ClientMessage,
};

use channel::{
    LspChannels,
};
use thread::lsp_thread;

pub mod channel;
mod thread;
mod client;
mod types;
mod server;
mod log;

pub use {
    thread::{
        Error,
    },
    server::{
        Server,
        SpecialBehavior,
    },
};

pub fn run_lsp_thread(channels: LspChannels) -> io::Result<std::thread::JoinHandle<()>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()?;
    let handle = std::thread::spawn(move || {
        let result = runtime.block_on(lsp_thread(channels));
        if let Err(e) = result {
            aprintln!("lsp errored: {e:?}");
        }
    });
    Ok(handle)
}
