# Productivity Hub

A unified dashboard for managing a suite of productivity tools with shared configuration and a single installer.

Live website: https://sloganking.github.io/hub/

## Included Tools

| Tool | Description | Hotkey-Activated |
|------|-------------|------------------|
| **DeskTalk** | Voice-to-text transcription with push-to-talk | Yes |
| **Speak Selected** | Read selected text aloud using AI TTS | Yes |
| **Quick Assistant** | Voice-activated AI assistant | Yes |
| **Flatten String** | Flatten clipboard text (remove newlines) | Yes |
| **Typo Fix** | Fix typos in selected text using AI | Yes |
| **OCR Paste** | Extract text from clipboard images | Yes |

## Features

- **Single Installer**: Install all tools at once with one NSIS installer
- **Shared API Key**: Configure your OpenAI API key once, used by all tools
- **System Tray**: Central tray icon to manage all tools
- **Start/Stop Control**: Start and stop individual tools from the dashboard
- **Auto-Start**: Configure which tools start automatically with the Hub
- **Hotkey Registry**: View and manage hotkeys across all tools

## Building

### Prerequisites

- Rust toolchain (1.70+)
- Windows (primary target)

### Development Build

```powershell
# Build just the hub-dashboard
cargo build -p hub-dashboard

# Run the dashboard
cargo run -p hub-dashboard
```

### Full Release Build

Use the build script to compile all tools and create the installer:

```powershell
# Build everything (debug)
.\scripts\build-all.ps1

# Build everything (release with installer)
.\scripts\build-all.ps1 -Release
```

The installer will be created at:
```
target/release/bundle/nsis/Productivity Hub_x.x.x_x64-setup.exe
```

## Project Structure

```
hub/
├── Cargo.toml              # Workspace manifest
├── crates/
│   ├── hub-common/         # Shared configuration library
│   │   └── src/
│   │       ├── config.rs   # Configuration management
│   │       ├── hotkeys.rs  # Hotkey registry
│   │       └── tools.rs    # Tool definitions
│   └── hub-dashboard/      # Tauri dashboard application
│       ├── src/
│       │   ├── main.rs
│       │   ├── process_manager.rs
│       │   └── tauri_commands.rs
│       └── ui/dist/        # Frontend (HTML/CSS/JS)
├── tools/                  # Git submodules for each tool
│   ├── desk-talk/
│   ├── speak-selected/
│   ├── quick-assistant/
│   ├── flatten-string/
│   ├── typo-fix/
│   └── ocr-paste/
└── scripts/
    └── build-all.ps1       # Full build script
```

## Configuration

Configuration is stored in:
- Windows: `%APPDATA%\hub\productivity-hub\config.json`

The OpenAI API key is stored securely using the Windows Credential Manager.

## License

MIT License - see [LICENSE](LICENSE)
