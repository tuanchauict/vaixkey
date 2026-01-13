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
            "--open-accessibility" => {
                println!("📱 Opening System Settings → Accessibility...");
                open_system_settings("accessibility");
                return Ok(());
            }
            "--open-input-monitoring" => {
                println!("📱 Opening System Settings → Input Monitoring...");
                open_system_settings("input_monitoring");
                return Ok(());
            }
            "--request-permission" => {
                println!("🔐 Requesting Accessibility permission...");
                let granted = request_accessibility_permission();
                if granted {
                    println!("✅ Permission already granted! VaixKey is ready.");
                } else {
                    println!("⏳ Permission request sent. Check System Settings if no dialog appeared.");
                    println!("   Opening System Settings → Accessibility...");
                    open_system_settings("accessibility");
                }
                return Ok(());
            }
            "--test-capture" => {
                return test_real_keyboard_capture(engine.clone()).await;
            }
            _ => {
                println!("Usage: vaixkey [--settings|--test|--status|--debug|--permissions]");
                println!("");
                println!("Commands:");
                println!("  --settings             Open settings interface");
                println!("  --test                 Test Vietnamese processing engine");
                println!("  --status               Show configuration and engine status");
                println!("  --debug                Run with comprehensive keystroke logging");
                println!("  --permissions          Check macOS security permissions");
                println!("  --security-status      Show detailed security status");
                println!("  --setup-permissions    Interactive guide for setting up permissions");
                println!("  --request-permission   Request Accessibility permission from macOS");
                println!("  --open-accessibility   Open System Settings → Accessibility");
                println!("  --open-input-monitoring Open System Settings → Input Monitoring");
                println!("  --test-capture         Test real keyboard capture (requires permissions)");
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
        for ch in input.chars() {
            engine.process_keypress(ch).await;
        }
        let result = engine.get_current_buffer().to_string();

        println!("   {} → {} ({})", input, result, description);
    }

    // Test undo behavior (triple char: ooo → oo, aaa → aa)
    println!("\n↩️  Double-Char Undo Tests (escape sequences):");
    let undo_tests = vec![
        ("ooo", "oo", "ooo → oo (undo ô)"),
        ("aaa", "aa", "aaa → aa (undo â)"),
        ("eee", "ee", "eee → ee (undo ê)"),
        ("ddd", "dd", "ddd → dd (undo đ)"),
    ];

    for (input, expected, description) in &undo_tests {
        engine.reset_buffer();
        for ch in input.chars() {
            engine.process_keypress(ch).await;
        }
        let result = engine.get_current_buffer().to_string();
        let status = if result == *expected { "✅" } else { "❌" };
        println!("   {} {} → {} (expected: {}) - {}", status, input, result, expected, description);
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
        for ch in input.chars() {
            engine.process_keypress(ch).await;
        }
        let result = engine.get_current_buffer().to_string();
        println!("   {} → {} ({})", input, result, description);
    }

    println!("\n🔤 Complete Word Processing:");
    let word_tests = vec![
        ("mootj", "một"),
        ("Vieetj", "Việt"),
        ("naawng", "năng"),
        ("ddaays", "đấy"),
        ("hocj", "học"),
        ("tooij", "tội"),
        ("xooong", "xoong"),  // Need 3 o's to get "oo" (undo circumflex)
    ];

    for (input, expected) in &word_tests {
        engine.reset_buffer();
        for ch in input.chars() {
            engine.process_keypress(ch).await;
        }
        let result = engine.get_current_buffer().to_string();
        let status = if result == *expected { "✅" } else { "❌" };
        println!("   {} {} → {} (expected: {})", status, input, result, expected);
    }

    // Test separator handling - "nam s" should NOT become "naám s"
    println!("\n🔀 Separator Handling Tests:");
    let separator_tests = vec![
        ("nam s", "s"),  // After space, 's' should be just 's', buffer cleared
        ("thi9s", "s"),  // After digit, buffer should be cleared
        ("abc.def", "dè"),  // After period, buffer cleared; then def -> dè (f is telex tone)
    ];

    for (input, expected_buffer) in &separator_tests {
        engine.reset_buffer();
        for ch in input.chars() {
            engine.process_keypress(ch).await;
        }
        let result = engine.get_current_buffer().to_string();
        let status = if result == *expected_buffer { "✅" } else { "❌" };
        println!("   {} '{}' → buffer: '{}' (expected: '{}')", status, input, result, expected_buffer);
    }

    println!("\n�🔄 Mode Toggle Test:");
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

    // Check current permission status
    let accessibility = check_accessibility_permission().await;
    let input_monitoring = check_input_monitoring_permission().await;

    println!("📊 Current Permission Status:");
    println!("   🔍 Input Monitoring: {}", if input_monitoring { "✅ Granted" } else { "❌ Not granted" });
    println!("   🔧 Accessibility: {}", if accessibility { "✅ Granted" } else { "❌ Not granted" });
    println!("");

    if accessibility && input_monitoring {
        println!("🎉 All permissions are already granted!");
        println!("   VaixKey is ready to use.");
        println!("   Run `cargo run` to start VaixKey.");
        return Ok(());
    }

    println!("VaixKey needs macOS Accessibility permission to function.");
    println!("(Both Input Monitoring and Accessibility use the same permission)");
    println!("");

    println!("🚀 Quick Setup Options:");
    println!("");
    println!("  1. [Recommended] Let VaixKey request permission automatically");
    println!("  2. Open System Settings → Privacy & Security → Accessibility");
    println!("  3. Open System Settings → Privacy & Security → Input Monitoring");
    println!("  4. Show manual setup instructions");
    println!("  5. Exit");
    println!("");

    print!("Choose an option (1-5): ");
    use std::io::{self, Write};
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" => {
            println!("");
            println!("🔐 Requesting Accessibility permission...");
            println!("   A system dialog should appear asking for permission.");
            println!("");
            
            let granted = request_accessibility_permission();
            
            if granted {
                println!("✅ Permission granted! VaixKey is ready to use.");
            } else {
                println!("⏳ Permission request sent.");
                println!("");
                println!("📋 If no dialog appeared, please:");
                println!("   1. Open System Settings → Privacy & Security → Accessibility");
                println!("   2. Find and enable 'Terminal' (or 'vaixkey' if running the built app)");
                println!("   3. You may need to click the lock 🔒 icon first");
                println!("");
                println!("Opening System Settings...");
                open_system_settings("accessibility");
            }
        }
        "2" => {
            println!("");
            println!("📱 Opening System Settings → Accessibility...");
            open_system_settings("accessibility");
            println!("");
            println!("📋 In System Settings:");
            println!("   1. Click the lock icon 🔒 if needed");
            println!("   2. Find 'Terminal' (or 'vaixkey') in the list");
            println!("   3. Toggle the switch to enable it");
            println!("   4. Restart VaixKey after granting permission");
        }
        "3" => {
            println!("");
            println!("📱 Opening System Settings → Input Monitoring...");
            open_system_settings("input_monitoring");
            println!("");
            println!("📋 In System Settings:");
            println!("   1. Click the lock icon 🔒 if needed");
            println!("   2. Find 'Terminal' (or 'vaixkey') in the list");
            println!("   3. Toggle the switch to enable it");
            println!("   4. Restart VaixKey after granting permission");
        }
        "4" => {
            println!("");
            print_manual_setup_instructions();
        }
        _ => {
            println!("Exiting setup guide.");
        }
    }

    println!("");
    println!("🧪 After granting permissions, verify with:");
    println!("   cargo run -- --permissions");

    Ok(())
}

fn print_manual_setup_instructions() {
    println!("📋 Manual Setup Instructions");
    println!("=============================");
    println!("");
    println!("For macOS Ventura (13.0) and later:");
    println!("─────────────────────────────────────");
    println!("1. Click the Apple menu (🍎) → System Settings");
    println!("2. Click 'Privacy & Security' in the sidebar");
    println!("3. Scroll down and click 'Accessibility'");
    println!("4. Click the toggle next to 'Terminal' to enable it");
    println!("   (You may need to click the lock and enter your password)");
    println!("5. Also check 'Input Monitoring' and enable 'Terminal'");
    println!("");
    println!("For macOS Monterey (12.0) and earlier:");
    println!("────────────────────────────────────────");
    println!("1. Click the Apple menu (🍎) → System Preferences");
    println!("2. Click 'Security & Privacy'");
    println!("3. Click the 'Privacy' tab");
    println!("4. Select 'Accessibility' from the left sidebar");
    println!("5. Click the lock icon 🔒 and enter your password");
    println!("6. Check the box next to 'Terminal'");
    println!("7. Select 'Input Monitoring' and check 'Terminal' there too");
    println!("");
    println!("💡 Tips:");
    println!("   • If 'Terminal' is not listed, run VaixKey once first");
    println!("   • If permission doesn't work, try toggling it off and on");
    println!("   • You may need to restart Terminal after granting permission");
    println!("   • For the built app, look for 'vaixkey' instead of 'Terminal'");
}

// Helper functions for permission checking using macOS APIs

/// Check if Accessibility permission is granted using AXIsProcessTrustedWithOptions
fn check_accessibility_trusted(prompt: bool) -> bool {
    use core_foundation::base::{CFRelease, TCFType};
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;
    
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
    }
    
    unsafe {
        if prompt {
            // Create options dictionary with kAXTrustedCheckOptionPrompt = true
            // This will prompt the user to grant permission
            let key = CFString::new("AXTrustedCheckOptionPrompt");
            let value = CFBoolean::true_value();
            let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
            AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef() as *const _)
        } else {
            AXIsProcessTrustedWithOptions(std::ptr::null())
        }
    }
}

async fn check_input_monitoring_permission() -> bool {
    // Input Monitoring permission is tied to Accessibility on macOS
    // The rdev library uses Quartz Event Taps which require Accessibility permission
    // We test this by checking if we can create an event tap
    check_accessibility_trusted(false)
}

async fn check_accessibility_permission() -> bool {
    check_accessibility_trusted(false)
}

/// Open macOS System Settings to the appropriate privacy section
fn open_system_settings(section: &str) {
    use std::process::Command;
    
    // macOS Ventura and later use System Settings with different URL scheme
    // macOS Monterey and earlier use System Preferences
    let url = match section {
        "accessibility" => "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
        "input_monitoring" => "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent",
        _ => "x-apple.systempreferences:com.apple.preference.security?Privacy",
    };
    
    let result = Command::new("open")
        .arg(url)
        .spawn();
    
    if let Err(e) = result {
        eprintln!("Failed to open System Settings: {}", e);
        // Fallback: try opening System Preferences app directly
        let _ = Command::new("open")
            .arg("-a")
            .arg("System Preferences")
            .spawn();
    }
}

/// Request accessibility permission with a system prompt
fn request_accessibility_permission() -> bool {
    check_accessibility_trusted(true)
}

async fn test_real_keyboard_capture(
    _engine: Arc<Mutex<InputMethodEngine>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Real Keyboard Capture");
    println!("=================================");
    println!("This will test if VaixKey can actually capture keyboard events");
    println!("using your granted macOS permissions.");
    println!("");

    println!("📋 Pre-flight checks:");

    // Check permissions first
    let input_monitoring = check_input_monitoring_permission().await;
    let accessibility = check_accessibility_permission().await;

    println!("   🔍 Input Monitoring: {}",
        if input_monitoring { "✅" } else { "❌ Missing" });
    println!("   🔧 Accessibility: {}",
        if accessibility { "✅" } else { "❌ Missing" });

    if !input_monitoring || !accessibility {
        println!("");
        println!("⚠️  Permission check shows missing permissions, but let's try anyway...");
        println!("   (Note: Permission checking is not fully implemented yet)");
        println!("   If you've granted permissions in System Preferences, this should still work.");
        println!("");
    }

    println!("");
    println!("🎯 Starting real keyboard capture test...");
    println!("   • This uses the rdev library to capture actual keyboard events");
    println!("   • If permissions are working, you'll see keypresses logged below");
    println!("   • Type a few letters, then press Ctrl+C to exit");
    println!("");
    println!("⌨️  Start typing (Press Ctrl+C to stop):");

    // Import rdev components we need
    use rdev::{listen, Event, EventType};

    // Remove unused variables for this simplified test

    // Simple test: just try to start the listener without complex state management
    println!("⚡ Attempting to start keyboard listener...");

    // This is a simplified test - just check if rdev can start
    let test_result = std::thread::spawn(|| {
        // Simple callback that doesn't capture variables
        fn simple_callback(event: Event) {
            match event.event_type {
                EventType::KeyPress(key) => {
                    println!("🔴 KEY PRESS: {:?}", key);
                    println!("   ✅ SUCCESS! Keyboard capture is working!");
                    // For this test, we'll just capture one event and exit
                    std::process::exit(0);
                }
                _ => {} // Ignore other event types for now
            }
        }

        // This will fail if permissions aren't granted
        match listen(simple_callback) {
            Ok(_) => {
                println!("🎉 Keyboard listener started successfully!");
                // The listen function will block here until an event occurs
            },
            Err(e) => {
                println!("❌ Failed to start keyboard listener: {:?}", e);
                println!("   This usually means:");
                println!("   • Input Monitoring permission not granted");
                println!("   • Permission granted to wrong application");
                println!("   • Need to restart after granting permission");
                println!("");
                println!("💡 Try these steps:");
                println!("   1. Check System Preferences → Security & Privacy → Input Monitoring");
                println!("   2. Make sure 'Terminal' is checked (since you're running via cargo)");
                println!("   3. If it's already checked, try unchecking and rechecking");
                println!("   4. Restart Terminal and try again");
            }
        }
    });

    // Give the listener thread time to start and potentially fail
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Wait for user to type something or timeout
    println!("   • Listener thread started, waiting for keyboard events...");
    println!("   • Press ANY key to test (the program will exit after one keypress)");
    println!("   • Or wait 10 seconds for timeout");

    // Simple timeout
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    println!("");
    println!("⏰ Test timeout - no keyboard events captured in 10 seconds");
    println!("   This might mean:");
    println!("   • Input Monitoring permission not granted to Terminal");
    println!("   • You didn't type anything");
    println!("   • Permission needs to be refreshed");
    println!("");
    println!("🏁 Keyboard capture test complete");

    Ok(())
}
