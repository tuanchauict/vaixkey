# VaixKey - Vietnamese Input Method for macOS

VaixKey is a modern Vietnamese input method for macOS, built with Rust. It provides support for Telex and VNI input methods, similar to UniKey but designed specifically for macOS with native integration.

## Features

- **Multiple Input Methods**: Support for Telex, VNI, and SimpleTelex
- **macOS Native**: Built specifically for macOS with proper system integration
- **Configurable Hotkeys**: Customize shortcuts for toggling Vietnamese mode and switching input methods
- **Status Bar Integration**: Visual indicator in the macOS menu bar
- **Settings GUI**: Native settings window built with Tauri
- **Unicode Support**: Proper Unicode normalization and Vietnamese text handling

## Project Structure

```
src/
├── main.rs                    # Main application entry point
├── config/                    # Configuration management
│   └── mod.rs                 # Config loading/saving, settings structure
├── input_method/              # Vietnamese input processing
│   ├── mod.rs                 # Input method engine and coordination
│   ├── vietnamese_engine.rs   # Core Vietnamese text transformation
│   ├── telex.rs              # Telex-specific processing
│   └── vni.rs                # VNI-specific processing
├── keyboard/                  # Keyboard monitoring and event handling
│   └── mod.rs                 # Key event capture and processing
└── gui/                       # Settings and status UI
    └── mod.rs                 # Tauri-based settings window
```

## Dependencies

- **rdev**: Cross-platform keyboard event monitoring
- **objc, cocoa, core-foundation, core-graphics**: macOS native APIs
- **unicode-normalization**: Proper Vietnamese text handling
- **tokio**: Async runtime for event processing
- **tauri**: Native GUI framework for settings
- **serde**: Configuration serialization
- **log, env_logger**: Logging support

## Building and Running

### Prerequisites

- Rust (latest stable version)
- macOS development environment
- Xcode command line tools

### Building

```bash
# Clone the repository
git clone <repository-url>
cd vaixkey

# Build the project
cargo build --release

# Run the application
cargo run
```

### Installation

```bash
# Build for release
cargo build --release

# The binary will be available at:
# target/release/vaixkey
```

## Configuration

VaixKey stores its configuration in `~/.config/vaixkey/config.toml`. The default configuration includes:

```toml
input_method = "Telex"
auto_start = false
show_status_bar = true

[hotkeys]
toggle_vietnamese = "Ctrl+Shift"
switch_input_method = "Ctrl+Alt+V"
```

## Input Methods

### Telex
- `aa` → `â`
- `aw` → `ă`
- `ee` → `ê`
- `oo` → `ô`
- `ow` → `ơ`
- `uw` → `ư`
- `dd` → `đ`
- `s` → sắc tone (á)
- `f` → huyền tone (à)
- `r` → hỏi tone (ả)
- `x` → ngã tone (ã)
- `j` → nặng tone (ạ)

### VNI
- `a6` → `ă`
- `a8` → `â`
- `e6` → `ê`
- `o6` → `ô`
- `o7` → `ơ`
- `u7` → `ư`
- `d9` → `đ`
- Numbers 1-5 for tone marks

## macOS Permissions

VaixKey requires the following macOS permissions:
- **Input Monitoring**: To capture keyboard events
- **Accessibility**: To inject processed text back into applications

These permissions will be requested when you first run the application.

## Development Status

This is the initial project setup. The core architecture is in place, but several components need implementation:

### TODO
- [ ] Implement macOS text injection APIs
- [ ] Complete Tauri GUI integration
- [ ] Add comprehensive Telex/VNI rule processing
- [ ] Implement status bar integration
- [ ] Add application-specific input handling
- [ ] Create installer package
- [ ] Add unit tests
- [ ] Performance optimization

### Current State
- ✅ Project structure and dependencies
- ✅ Configuration system
- ✅ Basic input method engine
- ✅ Vietnamese text transformation core
- ✅ Keyboard event monitoring framework
- ✅ GUI framework setup

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.