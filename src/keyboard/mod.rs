use crate::input_method::InputMethodEngine;
use log::{info, debug, error};
use rdev::{listen, Event, EventType, Key};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::mpsc;

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

// Global channel for sending events from the rdev callback
use std::sync::OnceLock;
static EVENT_SENDER: OnceLock<std::sync::Mutex<Option<mpsc::Sender<Event>>>> = OnceLock::new();

fn global_callback(event: Event) {
    if let Some(sender_mutex) = EVENT_SENDER.get() {
        if let Ok(guard) = sender_mutex.lock() {
            if let Some(sender) = guard.as_ref() {
                let _ = sender.send(event);
            }
        }
    }
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
            println!("🔍 Starting keyboard monitor with DEBUG logging enabled");
            println!("📝 Debug logs include: key events, buffer state, processing results");
        } else {
            info!("Starting keyboard monitor (set VAIXKEY_DEBUG=1 for detailed logging)");
        }

        println!("🎯 Real keyboard capture starting...");
        println!("⌨️  Type something to see keystroke capture in action!");
        println!("   Press Ctrl+C to exit\n");

        // Create channel for receiving events
        let (tx, rx) = mpsc::channel::<Event>();
        
        // Store sender in global state
        EVENT_SENDER.get_or_init(|| std::sync::Mutex::new(Some(tx)));
        
        // Start the rdev listener in a separate thread
        std::thread::spawn(move || {
            if let Err(e) = listen(global_callback) {
                eprintln!("❌ Failed to start keyboard listener: {:?}", e);
                eprintln!("");
                eprintln!("This usually means:");
                eprintln!("   • Accessibility permission not granted to Terminal");
                eprintln!("   • Need to restart Terminal after granting permission");
                eprintln!("");
                eprintln!("💡 Try: cargo run -- --open-accessibility");
            }
        });

        // Process events in the main async context
        let debug_mode = self.debug_mode;
        let engine = self.engine.clone();
        
        loop {
            // Check for events with a timeout to allow for graceful shutdown
            match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(event) => {
                    self.handle_event(event, debug_mode, &engine).await;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // No event, continue waiting
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    error!("Event channel disconnected");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_event(&self, event: Event, debug_mode: bool, engine: &Arc<Mutex<InputMethodEngine>>) {
        match event.event_type {
            EventType::KeyPress(key) => {
                let key_char = key_to_char(&key);
                let key_name = format!("{:?}", key);
                
                if debug_mode {
                    println!("🔴 KEY PRESS: {} ({})", 
                        key_name, 
                        key_char.map(|c| c.to_string()).unwrap_or_else(|| "special".to_string())
                    );
                }
                
                // Process the key if it's a character
                if let Some(ch) = key_char {
                    let mut eng = engine.lock().await;
                    let vietnamese_mode = eng.is_vietnamese_mode();
                    let buffer_before = eng.get_current_buffer().to_string();
                    
                    if debug_mode {
                        println!("📊 ENGINE STATE:");
                        println!("   Vietnamese Mode: {}", vietnamese_mode);
                        println!("   Buffer before: '{}'", buffer_before);
                    }
                    
                    if let Some(result) = eng.process_keypress(ch).await {
                        if debug_mode {
                            println!("🔤 Processing: '{}' → '{}'", ch, result);
                            if result != ch.to_string() {
                                println!("✨ Vietnamese transformation applied!");
                            }
                            println!("─────────────────────────────────────");
                        }
                    }
                } else if matches!(key, Key::Space | Key::Return) {
                    // Commit on space or enter
                    let mut eng = engine.lock().await;
                    let committed = eng.commit_current_text();
                    if debug_mode && !committed.is_empty() {
                        println!("📤 Committed: '{}'", committed);
                        println!("─────────────────────────────────────");
                    }
                }
            }
            EventType::KeyRelease(key) => {
                if debug_mode {
                    debug!("🔵 KEY RELEASE: {:?}", key);
                }
            }
            _ => {}
        }
    }
}

/// Convert rdev Key to a character
fn key_to_char(key: &Key) -> Option<char> {
    match key {
        Key::KeyA => Some('a'),
        Key::KeyB => Some('b'),
        Key::KeyC => Some('c'),
        Key::KeyD => Some('d'),
        Key::KeyE => Some('e'),
        Key::KeyF => Some('f'),
        Key::KeyG => Some('g'),
        Key::KeyH => Some('h'),
        Key::KeyI => Some('i'),
        Key::KeyJ => Some('j'),
        Key::KeyK => Some('k'),
        Key::KeyL => Some('l'),
        Key::KeyM => Some('m'),
        Key::KeyN => Some('n'),
        Key::KeyO => Some('o'),
        Key::KeyP => Some('p'),
        Key::KeyQ => Some('q'),
        Key::KeyR => Some('r'),
        Key::KeyS => Some('s'),
        Key::KeyT => Some('t'),
        Key::KeyU => Some('u'),
        Key::KeyV => Some('v'),
        Key::KeyW => Some('w'),
        Key::KeyX => Some('x'),
        Key::KeyY => Some('y'),
        Key::KeyZ => Some('z'),
        Key::Num0 => Some('0'),
        Key::Num1 => Some('1'),
        Key::Num2 => Some('2'),
        Key::Num3 => Some('3'),
        Key::Num4 => Some('4'),
        Key::Num5 => Some('5'),
        Key::Num6 => Some('6'),
        Key::Num7 => Some('7'),
        Key::Num8 => Some('8'),
        Key::Num9 => Some('9'),
        _ => None,
    }
}