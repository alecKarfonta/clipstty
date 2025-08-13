//! STT Clippy - Main Application Entry Point
//!
//! This is the main entry point for the STT Clippy desktop application.

use std::process;
use stt_clippy::{init, cleanup, Result};

#[tokio::main]
async fn main() {
    // Initialize the application
    if let Err(e) = init(None, None) {
        eprintln!("Failed to initialize STT Clippy: {}", e);
        process::exit(1);
    }

    // Run the main application
    if let Err(e) = run_app().await {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }

    // Cleanup before exit
    if let Err(e) = cleanup() {
        eprintln!("Failed to cleanup STT Clippy: {}", e);
        process::exit(1);
    }
}

/// Main application loop
async fn run_app() -> Result<()> {
    println!("STT Clippy v{} starting...", env!("CARGO_PKG_VERSION"));
    
    // TODO: Initialize services
    // TODO: Set up system tray
    // TODO: Register global hotkeys
    // TODO: Start audio processing
    // TODO: Run main event loop
    
    // For now, just wait a bit to simulate the app running
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    println!("STT Clippy started successfully!");
    
    // TODO: Keep the application running
    // This is a placeholder - in the real app, this would be the main event loop
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_app() {
        let result = run_app().await;
        assert!(result.is_ok());
    }
}
