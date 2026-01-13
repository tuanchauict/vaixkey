# VaixKey Usage Guide

## Running VaixKey

### Basic Usage
```bash
# Run VaixKey in background mode
cargo run

# Run VaixKey and open settings immediately
cargo run -- --settings
```

### Features Implemented

#### ✅ **Settings Interface**
- **Access**: Run with `--settings` flag or click settings in menu
- **Features**:
  - Input method selection (Telex, VNI, SimpleTelex)
  - Hotkey configuration display
  - Auto-start preferences
  - Notification settings
  - Modern macOS-style interface

#### ✅ **Native macOS Integration**
- **Status Notifications**: Native macOS notifications show mode changes
- **Browser-based Settings**: Opens in default browser with native styling
- **Configuration Management**: Persistent settings in `~/.config/vaixkey/config.toml`

#### ✅ **Vietnamese Input Methods**
- **Telex**: `aa` → `â`, `aw` → `ă`, `s` → sắc tone, etc.
- **VNI**: `a8` → `â`, `a6` → `ă`, number-based tones
- **SimpleTelex**: Simplified Telex without complex rules

#### ✅ **Configuration System**
- **Location**: `~/.config/vaixkey/config.toml`
- **Auto-creation**: Creates default config if none exists
- **Persistence**: Settings saved automatically
- **Hot-reload**: Changes take effect immediately

## Interface Tour

### Settings Window
The settings interface provides:

1. **Status Display**: Shows current Vietnamese/English mode
2. **Input Method Selection**:
   - Telex (default)
   - VNI
   - SimpleTelex
3. **Hotkey Configuration**:
   - Toggle Vietnamese Mode: `Ctrl+Shift`
   - Switch Input Method: `Ctrl+Alt+V`
4. **Preferences**:
   - Auto-start at login
   - Show status notifications

### Notifications
- Mode changes trigger native macOS notifications
- Shows "VaixKey: Vietnamese Mode" or "VaixKey: English Mode"
- Integrates with macOS notification center

## Development Status

### ✅ **Completed**
- Native macOS GUI implementation
- Configuration system with TOML persistence
- Vietnamese text transformation engine
- Settings interface with responsive design
- Native notification integration
- Command-line interface

### 🚧 **In Progress**
- System tray/menu bar integration
- Actual keyboard event capture
- Text injection into applications
- Hotkey registration and handling

### 📋 **Planned**
- Full keyboard monitoring implementation
- Application-specific input handling
- Advanced Telex/VNI rule processing
- Installer package creation
- Auto-updater functionality

## Technical Architecture

### Core Components
- **Config Module**: TOML-based configuration management
- **Input Method Engine**: Vietnamese text transformation
- **GUI Manager**: Native macOS interface
- **Keyboard Monitor**: Event capture (simulation mode)

### File Structure
```
src/
├── main.rs              # Application entry point
├── config/mod.rs        # Configuration management
├── input_method/        # Vietnamese processing engine
├── keyboard/mod.rs      # Event monitoring
└── gui/mod.rs          # Native GUI implementation
```

### Dependencies
- **Core**: `tokio`, `serde`, `log`
- **macOS**: `cocoa`, `objc`, `core-foundation`, `core-graphics`
- **Text**: `unicode-normalization`
- **Config**: `toml`, `serde_json`

## Building and Installation

### Development Build
```bash
cargo build
```

### Release Build
```bash
cargo build --release
```

### Run Tests
```bash
cargo test
```

The application is ready for basic GUI testing and configuration management. The core Vietnamese input processing is implemented and ready for integration with actual keyboard monitoring.