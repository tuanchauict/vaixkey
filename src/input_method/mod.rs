use crate::config::{Config, InputMethodType};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod telex;
pub mod vni;
pub mod vietnamese_engine;
pub mod unikey_engine;

pub use unikey_engine::{UnikeyEngine, ProcessResult, InputMethod};

#[derive(Debug)]
pub struct InputMethodEngine {
    #[allow(dead_code)]
    config: Arc<Mutex<Config>>,
    unikey_engine: UnikeyEngine,
    is_vietnamese_mode: bool,
    current_buffer: String,
}

impl InputMethodEngine {
    pub fn new(config: Arc<Mutex<Config>>) -> Self {
        Self {
            config,
            unikey_engine: UnikeyEngine::new(),
            is_vietnamese_mode: true, // Start in Vietnamese mode
            current_buffer: String::new(),
        }
    }

    pub async fn process_keypress(&mut self, key_char: char) -> ProcessResult {
        // Update the engine's Vietnamese mode
        if self.unikey_engine.is_vietnamese_mode() != self.is_vietnamese_mode {
            self.unikey_engine.set_vietnamese_mode(self.is_vietnamese_mode);
        }

        // Update input method from config
        let config = self.config.lock().await;
        let input_method = match config.input_method {
            InputMethodType::Telex | InputMethodType::SimpleTelex => InputMethod::Telex,
            InputMethodType::Vni => InputMethod::Vni,
        };
        drop(config);
        self.unikey_engine.set_input_method(input_method);

        // Process the keypress
        let result = self.unikey_engine.process(key_char);

        // Update current buffer based on result
        match &result {
            ProcessResult::PassThrough(c) => {
                self.current_buffer.push(*c);
            }
            ProcessResult::Output(_) | ProcessResult::Replace { .. } => {
                self.current_buffer = self.unikey_engine.get_buffer();
            }
        }

        result
    }

    pub fn toggle_vietnamese_mode(&mut self) {
        self.is_vietnamese_mode = !self.is_vietnamese_mode;
        self.unikey_engine.set_vietnamese_mode(self.is_vietnamese_mode);
        self.reset_buffer();
    }

    pub fn reset_buffer(&mut self) {
        self.current_buffer.clear();
        self.unikey_engine.clear_buf();
    }

    pub fn is_vietnamese_mode(&self) -> bool {
        self.is_vietnamese_mode
    }

    pub fn backspace(&mut self) -> Option<String> {
        // Process backspace through the engine
        let result = self.unikey_engine.process('\x08');
        match result {
            ProcessResult::PassThrough(_) => {
                self.current_buffer = self.unikey_engine.get_buffer();
                if self.current_buffer.is_empty() {
                    None
                } else {
                    Some(self.current_buffer.clone())
                }
            }
            ProcessResult::Output(text) => Some(text),
            ProcessResult::Replace { backspaces: _, text } => Some(text),
        }
    }

    pub fn commit_current_text(&mut self) -> String {
        let result = self.unikey_engine.get_buffer();
        self.reset_buffer();
        result
    }

    pub fn get_current_buffer(&self) -> &str {
        &self.current_buffer
    }
}