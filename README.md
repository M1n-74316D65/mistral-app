# Le Chat - Mistral AI Desktop App

A native desktop wrapper for [Le Chat](https://chat.mistral.ai/chat) (Mistral AI's chat interface) with a quick launcher feature inspired by Claude Desktop.

## Features

- **Native Desktop App**: Full-featured desktop application for Windows and macOS
- **Quick Launcher**: Press `Alt+Space` (Windows) or `Option+Space` (macOS) to open a spotlight-style quick input
- **System Tray**: App runs in the background with a tray icon for quick access
- **Dark/Light Mode**: Automatically matches your system theme
- **External Links**: Links in chat open in your default browser

## Installation

### macOS
1. Download `Le Chat_1.0.0_aarch64.dmg` from the releases
2. Open the DMG and drag `Le Chat.app` to your Applications folder
3. Launch from Applications

### Windows
1. Download the installer from the releases
2. Run the installer
3. Launch from Start Menu

## Usage

### Quick Launcher
- Press `Alt+Space` (Windows) or `Option+Space` (macOS) to open the launcher
- Type your message and press `Enter` to send
- Press `Escape` to close the launcher

### Main Window
- Click the tray icon to show the main window
- Use the full Mistral AI chat interface
- Close the window to hide to tray (app keeps running)

### System Tray Menu
- **Show Le Chat**: Open the main chat window
- **Quick Ask...**: Open the launcher
- **Quit**: Exit the application

## Development

### Prerequisites
- [Bun](https://bun.sh) (v1.0+)
- [Rust](https://rustup.rs) (v1.70+)
- Platform-specific dependencies:
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio Build Tools with C++ workload

### Setup
```bash
# Install dependencies
bun install

# Run in development mode
bun run dev

# Build for production
bun run build
```

### Project Structure
```
mistral-app/
├── src/                      # Frontend files
│   ├── launcher.html         # Quick launcher UI
│   ├── launcher.css          # Launcher styles
│   ├── launcher.js           # Launcher logic
│   └── index.html            # Fallback page
├── src-tauri/                # Tauri (Rust) backend
│   ├── src/
│   │   ├── lib.rs            # Main app logic
│   │   └── main.rs           # Entry point
│   ├── Cargo.toml            # Rust dependencies
│   ├── tauri.conf.json       # Tauri configuration
│   ├── capabilities/         # Permission configs
│   └── icons/                # App icons
├── package.json
└── README.md
```

### Build Outputs
- **macOS**: `src-tauri/target/release/bundle/macos/Le Chat.app`
- **macOS DMG**: `src-tauri/target/release/bundle/dmg/Le Chat_1.0.0_aarch64.dmg`
- **Windows**: `src-tauri/target/release/bundle/msi/Le Chat_1.0.0_x64.msi`

## Configuration

### Changing the Hotkey
Edit `src-tauri/src/lib.rs` and modify the shortcut in `setup_global_shortcut()`:
```rust
let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
```

### Window Sizes
Edit `src-tauri/tauri.conf.json` to change default window dimensions.

## Tech Stack

- **[Tauri v2](https://tauri.app)**: Desktop app framework
- **[Rust](https://rust-lang.org)**: Backend logic
- **[Bun](https://bun.sh)**: JavaScript runtime and package manager
- **Vanilla HTML/CSS/JS**: Lightweight frontend

## License

MIT

## Credits

- [Mistral AI](https://mistral.ai) for Le Chat
- [Tauri](https://tauri.app) for the framework
