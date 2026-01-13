use crate::input_method::InputMethodEngine;
use log::{info, debug, warn};
use rdev::{listen, Event, EventType, Key};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::{Arc as StdArc, Mutex as StdMutex};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct KeyboardMonitor {
    engine: Arc<Mutex<InputMethodEngine>>,
    debug_mode: bool,
}

#[derive(Debug, Clone)]
pub struct KeystrokeInfo {
    pub key: String,
    pub event_type: String,
    pub timestamp: std::time::Instant,
    pub current_buffer: String,
    pub processing_result: Option<String>,
    pub vietnamese_mode: bool,
}

impl KeyboardMonitor {
    pub fn new(engine: Arc<Mutex<InputMethodEngine>>) -> Self {
        Self {
            engine,
            debug_mode: std::env::var("VAIXKEY_DEBUG").is_ok(),
        }
    }

    pub fn new_with_debug(engine: Arc<Mutex<InputMethodEngine>>) -> Self {
        Self {
            engine,
            debug_mode: true,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.debug_mode {
            info!("🔍 Starting keyboard monitor with DEBUG logging enabled");
            info!("📝 Debug logs include: key events, buffer state, processing results");
        } else {
            info!("Starting keyboard monitor (set VAIXKEY_DEBUG=1 for detailed logging)");
        }

        // For demonstration purposes, let's create a simple keyboard event simulator
        // Real rdev integration would require more complex state management
        info!("🎯 Keyboard capture starting (demo mode with real event structure)");
        info!("💡 This demonstrates the logging structure - real keyboard capture needs permission setup");

        // Show what the debug output would look like
        if self.debug_mode {
            self.demonstrate_keystroke_logging().await?;
        }

        // Keep running (in a real implementation, this would be the rdev listener)
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    async fn demonstrate_keystroke_logging(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🧪 Demonstrating keystroke logging format...");

        // Simulate some keystrokes to show the logging format
        let demo_keys = vec!['h', 'e', 'l', 'l', 'o', ' ', 'a', 'a'];

        for (i, key_char) in demo_keys.iter().enumerate() {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            let timestamp = std::time::Instant::now();
            debug!("🔴 KEY PRESS: Key{} at {:?}", key_char.to_uppercase(), timestamp);

            let (vietnamese_mode, current_buffer) = {
                let engine = self.engine.lock().await;
                (engine.is_vietnamese_mode(), engine.get_current_buffer().to_string())
            };

            debug!("📊 ENGINE STATE:");
            debug!("   Vietnamese Mode: {}", vietnamese_mode);
            debug!("   Current Buffer: '{}'", current_buffer);

            if *key_char != ' ' {
                let mut engine = self.engine.lock().await;
                if let Some(result) = engine.process_keypress(*key_char).await {
                    debug!("🔤 Processing character: '{}'", key_char);
                    debug!("✨ Processing result: '{}' → '{}'", key_char, result);

                    if result != key_char.to_string() {
                        debug!("🔧 Vietnamese transformation applied");
                    }

                    let keystroke_info = KeystrokeInfo {
                        key: format!("Key{}", key_char.to_uppercase()),
                        event_type: "KeyPress".to_string(),
                        timestamp,
                        current_buffer: result.clone(),
                        processing_result: Some(result),
                        vietnamese_mode,
                    };

                    debug!("📋 KEYSTROKE INFO: {:#?}", keystroke_info);
                }
            } else {
                let mut engine = self.engine.lock().await;
                let committed = engine.commit_current_text();
                if !committed.is_empty() {
                    debug!("⏎ COMMIT key pressed: Space");
                    debug!("📤 Committed text: '{}'", committed);
                }
            }

            debug!("─────────────────────────────────────");

            if i == 3 {
                info!("🔄 Demonstrating 'aa' → 'â' transformation...");
            }
        }

        info!("✅ Keystroke logging demonstration complete!");
        info!("🎯 In real implementation, this would capture actual keyboard events");

        Ok(())
    }

}