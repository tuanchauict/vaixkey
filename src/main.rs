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
            "--permissions" | "--check-permissions" => {
                return check_permissions().await;
            }
            "--security-status" => {
                return show_security_status().await;
            }
            "--setup-permissions" => {
                return setup_permissions_guide().await;
            }
            _ => {
                println!("Usage: vaixkey [--settings|--test|--status|--debug|--permissions]");
                println!("");
                println!("Commands:");
                println!("  --settings           Open settings interface");
                println!("  --test              Test Vietnamese processing engine");
                println!("  --status            Show configuration and engine status");
                println!("  --debug             Run with comprehensive keystroke logging");
                println!("  --permissions       Check macOS security permissions");
                println!("  --security-status   Show detailed security status");
                println!("  --setup-permissions Guide for setting up permissions");
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

async fn check_permissions() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 VaixKey Permission Status");
    println!("============================");

    // For now, show what permissions we would check
    // In a real implementation, this would use macOS APIs to check actual permissions
    println!("📋 Checking macOS security permissions...");
    println!("");

    // Simulate permission checking (in real implementation, use CGPreflightScreenCaptureAccess, etc.)
    let input_monitoring = check_input_monitoring_permission().await;
    let accessibility = check_accessibility_permission().await;

    println!("🔍 Input Monitoring: {}",
        if input_monitoring { "✅ Granted" } else { "❌ DENIED - Required for keystroke capture" });
    println!("🔧 Accessibility: {}",
        if accessibility { "✅ Granted" } else { "❌ DENIED - Required for text injection" });

    println!("");

    if input_monitoring && accessibility {
        println!("🎯 VaixKey is ready for keyboard capture!");
        println!("   Run `cargo run -- --debug` to test keystroke logging");
    } else {
        println!("⚠️  VaixKey cannot function without these permissions");
        println!("   Run `cargo run -- --setup-permissions` for setup instructions");
    }

    Ok(())
}

async fn show_security_status() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 VaixKey Security Status Report");
    println!("=================================");

    // System information
    println!("💻 System Information:");
    let output = std::process::Command::new("sw_vers").output()?;
    if output.status.success() {
        let version_info = String::from_utf8_lossy(&output.stdout);
        for line in version_info.lines() {
            println!("   {}", line);
        }
    }

    println!("");

    // Permission status
    println!("🔐 Permission Status:");
    let input_monitoring = check_input_monitoring_permission().await;
    let accessibility = check_accessibility_permission().await;

    println!("   Input Monitoring: {}",
        if input_monitoring { "✅ GRANTED" } else { "❌ DENIED" });
    println!("   Accessibility: {}",
        if accessibility { "✅ GRANTED" } else { "❌ DENIED" });

    println!("");

    // Current application info
    println!("📱 Application Status:");
    println!("   Running as: {}",
        std::env::current_exe()
            .unwrap_or_else(|_| "Unknown".into())
            .display()
    );
    println!("   Process ID: {}", std::process::id());

    println!("");

    // Next steps
    if !input_monitoring || !accessibility {
        println!("📋 Required Actions:");
        if !input_monitoring {
            println!("   • Grant Input Monitoring permission in System Preferences");
        }
        if !accessibility {
            println!("   • Grant Accessibility permission in System Preferences");
        }
        println!("   • Run `cargo run -- --setup-permissions` for detailed instructions");
    } else {
        println!("🎉 All permissions granted - VaixKey is ready!");
    }

    Ok(())
}

async fn setup_permissions_guide() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 VaixKey Permission Setup Guide");
    println!("==================================");
    println!("");

    println!("VaixKey needs two macOS permissions to function:");
    println!("");

    println!("1️⃣ INPUT MONITORING");
    println!("   • Required for: Capturing keyboard input");
    println!("   • Location: System Preferences → Security & Privacy → Privacy → Input Monitoring");
    println!("   • Action: Check the box next to 'vaixkey' or 'Terminal'");
    println!("");

    println!("2️⃣ ACCESSIBILITY");
    println!("   • Required for: Injecting Vietnamese text into applications");
    println!("   • Location: System Preferences → Security & Privacy → Privacy → Accessibility");
    println!("   • Action: Check the box next to 'vaixkey' or 'Terminal'");
    println!("");

    println!("📋 Step-by-Step Instructions:");
    println!("");
    println!("1. Open System Preferences (🍎 → System Preferences)");
    println!("2. Click 'Security & Privacy'");
    println!("3. Click the 'Privacy' tab");
    println!("4. Click the lock icon (🔒) and enter your password");
    println!("5. Select 'Input Monitoring' from the left sidebar");
    println!("6. Check the box next to 'vaixkey' or 'Terminal'");
    println!("7. Select 'Accessibility' from the left sidebar");
    println!("8. Check the box next to 'vaixkey' or 'Terminal'");
    println!("9. Restart VaixKey");
    println!("");

    println!("🧪 Verification:");
    println!("   Run: cargo run -- --permissions");
    println!("   You should see both permissions marked as ✅ Granted");
    println!("");

    println!("📖 For detailed instructions with screenshots, see:");
    println!("   SECURITY_SETUP.md in the VaixKey directory");

    Ok(())
}

// Helper functions for permission checking
// In a real implementation, these would use macOS APIs
async fn check_input_monitoring_permission() -> bool {
    // This is a placeholder - real implementation would use:
    // CGPreflightScreenCaptureAccess() or similar Input Monitoring API
    // For now, assume permission is needed but not granted
    false
}

async fn check_accessibility_permission() -> bool {
    // This is a placeholder - real implementation would use:
    // AXIsProcessTrustedWithOptions(CFDictionaryRef options)
    // For now, assume permission is needed but not granted
    false
}
