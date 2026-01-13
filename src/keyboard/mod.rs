use crate::input_method::{InputMethodEngine, ProcessResult};
use log::{info, debug, error};
use rdev::{grab, simulate, Event, EventType, Key};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Flag to track if we're currently injecting text (to avoid feedback loop)
use std::sync::atomic::{AtomicBool, Ordering};
static INJECTING: AtomicBool = AtomicBool::new(false);

/// Struct to hold processing result from engine
#[derive(Clone, Debug)]
enum GrabAction {
    PassThrough,
    Block,
    BlockAndInject { backspaces: usize, text: String },
}

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

// Channel for communicating between grab callback and async processor
use std::sync::OnceLock;
static KEY_CHANNEL: OnceLock<(
    std::sync::Mutex<mpsc::Sender<(Key, bool)>>,  // key, is_press
    std::sync::Mutex<mpsc::Receiver<GrabAction>>,
)> = OnceLock::new();

static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

fn grab_callback(event: Event) -> Option<Event> {
    // If we're injecting, let all events through
    if INJECTING.load(Ordering::SeqCst) {
        return Some(event);
    }

    match event.event_type {
        EventType::KeyPress(key) => {
            // Always pass through modifier keys (Ctrl, Alt, Cmd, Shift)
            if is_modifier_key(&key) {
                return Some(event);
            }
            
            // Check if this is a separator key that should clear the buffer
            if is_separator_key(&key) {
                // Notify the engine to clear its buffer, but let the key through
                if let Some((tx_mutex, rx_mutex)) = KEY_CHANNEL.get() {
                    if let Ok(tx) = tx_mutex.lock() {
                        let _ = tx.send((key, true));
                    }
                    // Wait for response but always pass through
                    if let Ok(rx) = rx_mutex.lock() {
                        let _ = rx.recv_timeout(Duration::from_millis(50));
                    }
                }
                return Some(event);
            }
            
            // Check if this is a character key we should process
            if let Some(_ch) = key_to_char(&key) {
                // Send to processor and wait for response
                if let Some((tx_mutex, rx_mutex)) = KEY_CHANNEL.get() {
                    if let Ok(tx) = tx_mutex.lock() {
                        let _ = tx.send((key, true));
                    }
                    if let Ok(rx) = rx_mutex.lock() {
                        // Wait for action with timeout
                        match rx.recv_timeout(Duration::from_millis(50)) {
                            Ok(GrabAction::PassThrough) => {
                                return Some(event);
                            }
                            Ok(GrabAction::Block) => {
                                return None;
                            }
                            Ok(GrabAction::BlockAndInject { backspaces, text }) => {
                                // Block the event and inject our replacement
                                let debug = DEBUG_MODE.load(Ordering::SeqCst);
                                // Inject in a separate thread to not block
                                std::thread::spawn(move || {
                                    inject_replacement(backspaces, &text, debug);
                                });
                                return None;
                            }
                            Err(_) => {
                                // Timeout, let event through
                                return Some(event);
                            }
                        }
                    }
                }
            }
            Some(event)
        }
        EventType::KeyRelease(_key) => {
            // Let key releases through
            Some(event)
        }
        _ => Some(event),
    }
}

/// Check if this key is a modifier key that should always pass through
fn is_modifier_key(key: &Key) -> bool {
    matches!(key, 
        Key::ShiftLeft | Key::ShiftRight |
        Key::ControlLeft | Key::ControlRight |
        Key::Alt | Key::AltGr |
        Key::MetaLeft | Key::MetaRight |  // Command key on macOS
        Key::CapsLock | Key::NumLock | Key::ScrollLock |
        Key::Function  // Fn key
    )
}

/// Check if this key is a separator that should clear the buffer
fn is_separator_key(key: &Key) -> bool {
    matches!(key, 
        Key::Space | Key::Return | Key::Tab | 
        Key::Escape | Key::Backspace |
        Key::UpArrow | Key::DownArrow | Key::LeftArrow | Key::RightArrow |
        Key::Home | Key::End | Key::PageUp | Key::PageDown |
        Key::Delete | Key::Insert
    )
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
        DEBUG_MODE.store(self.debug_mode, Ordering::SeqCst);
        
        if self.debug_mode {
            println!("🔍 Starting keyboard monitor with DEBUG logging enabled");
            println!("📝 Debug logs include: key events, buffer state, processing results");
        } else {
            info!("Starting keyboard monitor (set VAIXKEY_DEBUG=1 for detailed logging)");
        }

        println!("🎯 Real keyboard GRAB starting (events will be intercepted)...");
        println!("⌨️  Type something to see keystroke capture in action!");
        println!("   Press Ctrl+C to exit\n");

        // Create bidirectional channels for communication with grab callback
        let (key_tx, key_rx) = mpsc::channel::<(Key, bool)>();
        let (action_tx, action_rx) = mpsc::channel::<GrabAction>();
        
        // Store channels in global state
        KEY_CHANNEL.get_or_init(|| (
            std::sync::Mutex::new(key_tx),
            std::sync::Mutex::new(action_rx),
        ));
        
        // Start the grab in a separate thread
        std::thread::spawn(move || {
            if let Err(e) = grab(grab_callback) {
                eprintln!("❌ Failed to start keyboard grab: {:?}", e);
                eprintln!("");
                eprintln!("This usually means:");
                eprintln!("   • Accessibility permission not granted to Terminal");
                eprintln!("   • Need to restart Terminal after granting permission");
                eprintln!("");
                eprintln!("💡 Try: cargo run -- --open-accessibility");
            }
        });

        // Process key events and send back actions
        let debug_mode = self.debug_mode;
        let engine = self.engine.clone();
        
        loop {
            // Check for key events with a timeout
            match key_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok((key, _is_press)) => {
                    // Check if it's a separator key
                    if is_separator_key(&key) {
                        // Clear the engine buffer
                        let mut eng = engine.lock().await;
                        if debug_mode {
                            println!("📤 Separator key, clearing buffer: '{}'", eng.get_current_buffer());
                            println!("─────────────────────────────────────");
                        }
                        eng.reset_buffer();
                        drop(eng);
                        let _ = action_tx.send(GrabAction::PassThrough);
                    } else if let Some(ch) = key_to_char(&key) {
                        let action = self.process_key(ch, debug_mode, &engine).await;
                        let _ = action_tx.send(action);
                    } else {
                        let _ = action_tx.send(GrabAction::PassThrough);
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    error!("Key channel disconnected");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn process_key(&self, ch: char, debug_mode: bool, engine: &Arc<Mutex<InputMethodEngine>>) -> GrabAction {
        let mut eng = engine.lock().await;
        let _vietnamese_mode = eng.is_vietnamese_mode();
        let buffer_before = eng.get_current_buffer().to_string();
        
        if debug_mode {
            println!("🔴 KEY: '{}' (buffer: '{}')", ch, buffer_before);
        }
        
        let result = eng.process_keypress(ch).await;
        drop(eng);
        
        match &result {
            ProcessResult::PassThrough(c) => {
                if debug_mode {
                    println!("🔤 Pass through: '{}'", c);
                    println!("─────────────────────────────────────");
                }
                GrabAction::PassThrough
            }
            ProcessResult::Output(text) => {
                if debug_mode {
                    println!("🔤 Output: '{}' (blocking original, injecting)", text);
                    println!("─────────────────────────────────────");
                }
                // Block the key and inject text
                GrabAction::BlockAndInject {
                    backspaces: 0,
                    text: text.clone(),
                }
            }
            ProcessResult::Replace { backspaces, text } => {
                if debug_mode {
                    println!("🔤 Replace: {} backspaces, then '{}'", backspaces, text);
                    println!("✨ Vietnamese transformation applied!");
                    println!("─────────────────────────────────────");
                }
                // Block the key, send backspaces, then inject new text
                GrabAction::BlockAndInject {
                    backspaces: *backspaces,
                    text: text.clone(),
                }
            }
        }
    }
}

/// Inject replacement text: send backspaces then type new text
fn inject_replacement(backspaces: usize, text: &str, debug_mode: bool) {
    if backspaces == 0 && text.is_empty() {
        return;
    }

    INJECTING.store(true, Ordering::SeqCst);
    
    // Small delay to let grab callback return
    thread::sleep(Duration::from_millis(5));
    
    // Send backspaces to delete the original characters
    for _ in 0..backspaces {
        send_key(Key::Backspace, debug_mode);
        thread::sleep(Duration::from_millis(2));
    }
    
    // Type the new text using Unicode input
    for c in text.chars() {
        send_unicode_char(c, debug_mode);
        thread::sleep(Duration::from_millis(2));
    }
    
    INJECTING.store(false, Ordering::SeqCst);
}

/// Send a single key press and release
fn send_key(key: Key, debug_mode: bool) {
    if let Err(e) = simulate(&EventType::KeyPress(key)) {
        if debug_mode {
            eprintln!("⚠️  Failed to simulate key press: {:?}", e);
        }
    }
    thread::sleep(Duration::from_millis(1));
    if let Err(e) = simulate(&EventType::KeyRelease(key)) {
        if debug_mode {
            eprintln!("⚠️  Failed to simulate key release: {:?}", e);
        }
    }
}

/// Send a Unicode character using the platform's Unicode input method
/// On macOS, we use the CGEvent API which rdev wraps
fn send_unicode_char(c: char, debug_mode: bool) {
    // For ASCII characters, we can use direct key simulation
    if c.is_ascii_alphabetic() {
        let key = char_to_key(c);
        if let Some(k) = key {
            let needs_shift = c.is_uppercase();
            if needs_shift {
                let _ = simulate(&EventType::KeyPress(Key::ShiftLeft));
                thread::sleep(Duration::from_millis(1));
            }
            send_key(k, debug_mode);
            if needs_shift {
                let _ = simulate(&EventType::KeyRelease(Key::ShiftLeft));
            }
            return;
        }
    }
    
    // For Vietnamese/Unicode characters, use platform-specific method
    #[cfg(target_os = "macos")]
    {
        send_unicode_char_macos(c, debug_mode);
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // Fallback: try rdev's Unknown key with the unicode value
        if debug_mode {
            eprintln!("⚠️  Unicode injection not fully implemented on this platform");
        }
    }
}

#[cfg(target_os = "macos")]
fn send_unicode_char_macos(c: char, debug_mode: bool) {
    use core_graphics::event::{CGEvent, CGEventTapLocation};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    
    // Create event source
    let source = match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
        Ok(s) => s,
        Err(_) => {
            if debug_mode {
                eprintln!("⚠️  Failed to create CGEventSource");
            }
            return;
        }
    };
    
    // Create a keyboard event
    let event = match CGEvent::new_keyboard_event(source.clone(), 0, true) {
        Ok(e) => e,
        Err(_) => {
            if debug_mode {
                eprintln!("⚠️  Failed to create CGEvent");
            }
            return;
        }
    };
    
    // Set the Unicode string for this event
    let chars: Vec<u16> = c.encode_utf16(&mut [0u16; 2]).to_vec();
    event.set_string_from_utf16_unchecked(&chars);
    
    // Post the event
    event.post(CGEventTapLocation::HID);
    
    // Send key up
    if let Ok(up_event) = CGEvent::new_keyboard_event(source, 0, false) {
        up_event.post(CGEventTapLocation::HID);
    }
    
    if debug_mode {
        debug!("📤 Injected Unicode: '{}'", c);
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

/// Convert a character to rdev Key (lowercase)
fn char_to_key(c: char) -> Option<Key> {
    match c.to_ascii_lowercase() {
        'a' => Some(Key::KeyA),
        'b' => Some(Key::KeyB),
        'c' => Some(Key::KeyC),
        'd' => Some(Key::KeyD),
        'e' => Some(Key::KeyE),
        'f' => Some(Key::KeyF),
        'g' => Some(Key::KeyG),
        'h' => Some(Key::KeyH),
        'i' => Some(Key::KeyI),
        'j' => Some(Key::KeyJ),
        'k' => Some(Key::KeyK),
        'l' => Some(Key::KeyL),
        'm' => Some(Key::KeyM),
        'n' => Some(Key::KeyN),
        'o' => Some(Key::KeyO),
        'p' => Some(Key::KeyP),
        'q' => Some(Key::KeyQ),
        'r' => Some(Key::KeyR),
        's' => Some(Key::KeyS),
        't' => Some(Key::KeyT),
        'u' => Some(Key::KeyU),
        'v' => Some(Key::KeyV),
        'w' => Some(Key::KeyW),
        'x' => Some(Key::KeyX),
        'y' => Some(Key::KeyY),
        'z' => Some(Key::KeyZ),
        '0' => Some(Key::Num0),
        '1' => Some(Key::Num1),
        '2' => Some(Key::Num2),
        '3' => Some(Key::Num3),
        '4' => Some(Key::Num4),
        '5' => Some(Key::Num5),
        '6' => Some(Key::Num6),
        '7' => Some(Key::Num7),
        '8' => Some(Key::Num8),
        '9' => Some(Key::Num9),
        _ => None,
    }
}