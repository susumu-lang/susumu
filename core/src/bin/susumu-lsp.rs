//! Susumu Language Server binary
//! 
//! Provides IDE features through the Language Server Protocol

use std::error::Error;

#[cfg(feature = "lsp")]
fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    env_logger::init();
    susumu::lsp::run_lsp_server()
}

#[cfg(not(feature = "lsp"))]
fn main() {
    eprintln!("LSP support not compiled in. Build with --features lsp");
    std::process::exit(1);
}