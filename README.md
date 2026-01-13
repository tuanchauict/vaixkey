# VaixKey - Vietnamese Input Method for macOS

VaixKey is a modern Vietnamese input method for macOS, built with Rust. It provides Telex input method support based on the UniKey/Uk362 algorithm, designed specifically for macOS with native integration.

## Features

- **Telex Input Method**: Full support for Vietnamese Telex typing
- **Real Keyboard Interception**: Uses `rdev::grab` to intercept and transform keystrokes in real-time
- **macOS Native**: Built specifically for macOS with proper system integration using CGEvent API
- **Smart Diphthong Handling**: Automatic "uo" → "ươ" transformation (e.g., `nguoiwf` → `người`)
- **Escape Sequences**: Type `ooo` to get `oo`, `aaa` to get `aa` (undo circumflex)
- **Proper Tone Placement**: Handles Q/GI prefix exceptions and consonant-after rules
- **Unicode Support**: Proper Unicode text injection via macOS CGEvent API

## Quick Start

```bash
# Build and run
cargo build --release
cargo run

# Run tests
cargo run -- --test

# Run with debug logging
cargo run -- --debug
```

## macOS Permissions

VaixKey requires **Accessibility** permission to capture and inject keystrokes:

1. Go to **System Preferences** → **Security & Privacy** → **Privacy** → **Accessibility**
2. Add your terminal or the VaixKey app to the allowed list
3. Restart VaixKey after granting permission

## Telex Input Guide

### Basic Characters
| Input | Output | Description |
|-------|--------|-------------|
| `aa` | `â` | a with circumflex |
| `aw` | `ă` | a with breve |
| `ee` | `ê` | e with circumflex |
| `oo` | `ô` | o with circumflex |
| `ow` | `ơ` | o with horn |
| `uw` | `ư` | u with horn |
| `dd` | `đ` | d with stroke |

### Tone Marks
| Input | Tone | Example |
|-------|------|---------|
| `s` | sắc (acute) | `as` → `á` |
| `f` | huyền (grave) | `af` → `à` |
| `r` | hỏi (hook) | `ar` → `ả` |
| `x` | ngã (tilde) | `ax` → `ã` |
| `j` | nặng (dot) | `aj` → `ạ` |

### Special Cases
| Input | Output | Description |
|-------|--------|-------------|
| `ooo` | `oo` | Undo circumflex (escape) |
| `nguoiwf` | `người` | Auto uo→ươ diphthong |
| `quas` | `quá` | Q prefix: tone on last vowel |
| `gias` | `giá` | GI prefix: tone on last vowel |
| `tuaans` | `tuấn` | Consonant after: tone on last vowel |

## Project Structure

```
src/
├── main.rs                    # Application entry, CLI, tests
├── config/mod.rs              # Configuration management
├── input_method/
│   ├── mod.rs                 # InputMethodEngine wrapper
│   └── unikey_engine.rs       # Core Unikey algorithm (~950 lines)
├── keyboard/mod.rs            # Keyboard grab and text injection
└── gui/mod.rs                 # GUI manager (placeholder)
```

## How It Works

1. **Keyboard Grab**: Uses `rdev::grab` with `unstable_grab` feature to intercept all keystrokes
2. **Vietnamese Processing**: Processes each key through `UnikeyEngine` based on Uk362 algorithm
3. **Text Injection**: Uses macOS `CGEvent` API with `set_string_from_utf16_unchecked` for Unicode injection
4. **Buffer Management**: Maintains a character buffer for context-aware transformations

## Development

### Dependencies
- `rdev` (0.5 with `unstable_grab`): Keyboard event interception
- `core-graphics`, `core-foundation`: macOS native APIs
- `tokio`: Async runtime
- `unicode-normalization`: Vietnamese text handling

### Running Tests
```bash
cargo run -- --test    # Run all engine tests
cargo run -- --debug   # Interactive debug mode with logging
cargo run -- --status  # Show configuration status
```

## Based On

This project's Vietnamese processing engine is based on the **UniKey/Uk362** algorithm by Pham Kim Long. The original C++ implementation has been ported to Rust with the following key functions:
- `process()`: Main key processing loop
- `double_char()`: Handle aa→â, oo→ô, dd→đ with undo
- `put_breve_mark()`: Handle aw→ă, ow→ơ, uw→ư with uo→ươ diphthong
- `put_tone_mark()`: Tone placement with Q/GI prefix exceptions

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.