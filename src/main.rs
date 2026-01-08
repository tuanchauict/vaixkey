mod config;
mod input_method;
mod keyboard;
mod gui;

use log::{info, error};
use std::sync::Arc;
use tokio::sync::Mutex;

use config::Config;
use input_method::InputMethodEngine;
use keyboard::KeyboardMonitor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    info!("Starting VaixKey Vietnamese Input Method");

    // Load configuration
    let config = Arc::new(Mutex::new(Config::load()?));
    info!("Configuration loaded successfully");

    // Initialize input method engine
    let engine = Arc::new(Mutex::new(InputMethodEngine::new(config.clone())));

    // Start keyboard monitor
    let keyboard_monitor = KeyboardMonitor::new(engine.clone());

    // Start the application
    info!("VaixKey is now running. Press Ctrl+C to exit.");

    // Run keyboard monitoring in background
    let monitor_handle = tokio::spawn(async move {
        if let Err(e) = keyboard_monitor.start().await {
            error!("Keyboard monitor error: {}", e);
        }
    });

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutting down VaixKey");

    monitor_handle.abort();
    Ok(())
}
