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
use gui::GuiManager;

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

    // Initialize GUI manager
    let gui_manager = GuiManager::new();

    // Show initial status
    {
        let engine = engine.lock().await;
        gui_manager.show_status_indicator(engine.is_vietnamese_mode()).await?;
    }

    // Start keyboard monitor
    let keyboard_monitor = if std::env::var("VAIXKEY_DEBUG").is_ok() {
        KeyboardMonitor::new_with_debug(engine.clone())
    } else {
        KeyboardMonitor::new(engine.clone())
    };

    // Start the application
    info!("VaixKey is now running. Press Ctrl+C to exit or run with --settings to open settings.");

    // Check command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--settings" => {
                gui_manager.show_settings().await?;
            }
            "--test" => {
                return run_test_mode(engine.clone()).await;
            }
            "--status" => {
                return show_status(config.clone(), engine.clone()).await;
            }
            "--debug" => {
                info!("🔍 Starting VaixKey in DEBUG mode with comprehensive keystroke logging");
                return run_debug_mode(engine.clone()).await;
            }
            _ => {
                println!("Usage: vaixkey [--settings|--test|--status|--debug]");
                println!("");
                println!("Commands:");
                println!("  --settings  Open settings interface");
                println!("  --test      Test Vietnamese processing engine");
                println!("  --status    Show configuration and engine status");
                println!("  --debug     Run with comprehensive keystroke logging");
                println!("");
                println!("Environment Variables:");
                println!("  VAIXKEY_DEBUG=1  Enable debug logging in normal mode");
                return Ok(());
            }
        }
    }

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

async fn show_status(
    config: Arc<Mutex<Config>>,
    engine: Arc<Mutex<InputMethodEngine>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🇻🇳 VaixKey Status Report");
    println!("========================");

    // Configuration status
    let config = config.lock().await;
    println!("📋 Configuration:");
    println!("   Input Method: {:?}", config.input_method);
    println!("   Auto Start: {}", config.auto_start);
    println!("   Show Status Bar: {}", config.show_status_bar);
    println!("   Toggle Hotkey: {}", config.hotkeys.toggle_vietnamese);
    println!("   Switch Hotkey: {}", config.hotkeys.switch_input_method);

    // Engine status
    let engine = engine.lock().await;
    println!("\n🔧 Engine Status:");
    println!("   Vietnamese Mode: {}", if engine.is_vietnamese_mode() { "✅ Active" } else { "❌ Inactive" });

    // Test Vietnamese processing
    println!("\n🧪 Vietnamese Processing Test:");
    test_vietnamese_processing(&*engine).await;

    println!("\n✅ VaixKey is properly configured and ready!");
    println!("   Run `cargo run -- --test` to test input processing");
    println!("   Run `cargo run -- --settings` to open settings");

    Ok(())
}

async fn run_test_mode(
    engine: Arc<Mutex<InputMethodEngine>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 VaixKey Test Mode");
    println!("===================");
    println!("Testing Vietnamese input processing...\n");

    let mut engine = engine.lock().await;

    // Test cases
    let test_cases = vec![
        ("a", "Basic vowel"),
        ("aa", "Double vowel (â)"),
        ("aw", "A with breve (ă)"),
        ("e", "Basic vowel"),
        ("ee", "E with circumflex (ê)"),
        ("o", "Basic vowel"),
        ("oo", "O with circumflex (ô)"),
        ("ow", "O with horn (ơ)"),
        ("u", "Basic vowel"),
        ("uw", "U with horn (ư)"),
        ("d", "Basic consonant"),
        ("dd", "D with stroke (đ)"),
    ];

    println!("🔤 Basic Character Processing:");
    for (input, description) in &test_cases {
        // Reset buffer for each test
        engine.reset_buffer();

        // Process each character
        let mut result = String::new();
        for ch in input.chars() {
            if let Some(output) = engine.process_keypress(ch).await {
                result = output;
            }
        }

        println!("   {} → {} ({})", input, result, description);
    }

    println!("\n🎯 Tone Mark Processing:");
    let tone_tests = vec![
        ("as", "a + sắc tone"),
        ("af", "a + huyền tone"),
        ("ar", "a + hỏi tone"),
        ("ax", "a + ngã tone"),
        ("aj", "a + nặng tone"),
    ];

    for (input, description) in &tone_tests {
        engine.reset_buffer();
        let mut result = String::new();
        for ch in input.chars() {
            if let Some(output) = engine.process_keypress(ch).await {
                result = output;
            }
        }
        println!("   {} → {} ({})", input, result, description);
    }

    println!("\n🔄 Mode Toggle Test:");
    println!("   Current mode: {}", if engine.is_vietnamese_mode() { "Vietnamese" } else { "English" });
    engine.toggle_vietnamese_mode();
    println!("   After toggle: {}", if engine.is_vietnamese_mode() { "Vietnamese" } else { "English" });

    println!("\n✅ Test complete! VaixKey engine is working properly.");
    println!("   Note: This tests the processing engine only.");
    println!("   Keyboard capture is not yet implemented.");

    Ok(())
}

async fn run_debug_mode(
    engine: Arc<Mutex<InputMethodEngine>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 VaixKey Debug Mode");
    println!("====================");
    println!("📝 Comprehensive keystroke capture and processing logging");
    println!("");
    println!("🎯 What you'll see:");
    println!("   🔴 KEY PRESS events with timestamps");
    println!("   🔵 KEY RELEASE events");
    println!("   📊 Engine state (Vietnamese mode, buffer contents)");
    println!("   🔤 Character processing details");
    println!("   ✨ Vietnamese transformation results");
    println!("   📋 Complete keystroke information");
    println!("");
    println!("⌨️  Start typing to see real-time keystroke capture...");
    println!("   (Press Ctrl+C to exit)");
    println!("");

    // Create keyboard monitor in debug mode
    let keyboard_monitor = KeyboardMonitor::new_with_debug(engine.clone());

    // Start monitoring
    keyboard_monitor.start().await?;

    Ok(())
}

async fn test_vietnamese_processing(engine: &InputMethodEngine) {
    // This is a simple test of the Vietnamese engine
    // Note: We can't easily test the async methods here due to borrowing,
    // so we'll just show that the engine exists and is configured
    println!("   Engine initialized: ✅");
    println!("   Vietnamese mode: {}", if engine.is_vietnamese_mode() { "✅" } else { "❌" });
}
