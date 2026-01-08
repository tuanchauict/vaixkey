use crate::config::{Config, InputMethodType};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod telex;
pub mod vni;
pub mod vietnamese_engine;

use vietnamese_engine::VietnameseEngine;

#[derive(Debug)]
pub struct InputMethodEngine {
    config: Arc<Mutex<Config>>,
    vietnamese_engine: VietnameseEngine,
    is_vietnamese_mode: bool,
    current_buffer: String,
}

impl InputMethodEngine {
    pub fn new(config: Arc<Mutex<Config>>) -> Self {
        Self {
            config,
            vietnamese_engine: VietnameseEngine::new(),
            is_vietnamese_mode: true, // Start in Vietnamese mode
            current_buffer: String::new(),
        }
    }

    pub async fn process_keypress(&mut self, key_char: char) -> Option<String> {
        if !self.is_vietnamese_mode {
            // Pass through directly if not in Vietnamese mode
            return Some(key_char.to_string());
        }

        // Add character to buffer
        self.current_buffer.push(key_char);

        // Process with Vietnamese engine
        let config = self.config.lock().await;
        match config.input_method {
            InputMethodType::Telex => {
                self.vietnamese_engine.process_telex(&self.current_buffer)
            }
            InputMethodType::Vni => {
                self.vietnamese_engine.process_vni(&self.current_buffer)
            }
            InputMethodType::SimpleTelex => {
                self.vietnamese_engine.process_simple_telex(&self.current_buffer)
            }
        }
    }

    pub fn toggle_vietnamese_mode(&mut self) {
        self.is_vietnamese_mode = !self.is_vietnamese_mode;
        self.reset_buffer();
    }

    pub fn reset_buffer(&mut self) {
        self.current_buffer.clear();
    }

    pub fn is_vietnamese_mode(&self) -> bool {
        self.is_vietnamese_mode
    }

    pub fn backspace(&mut self) -> Option<String> {
        if !self.current_buffer.is_empty() {
            self.current_buffer.pop();
            if self.current_buffer.is_empty() {
                None
            } else {
                // Reprocess the remaining buffer
                self.vietnamese_engine.process_telex(&self.current_buffer)
            }
        } else {
            None
        }
    }

    pub fn commit_current_text(&mut self) -> String {
        let result = self.current_buffer.clone();
        self.reset_buffer();
        result
    }
}