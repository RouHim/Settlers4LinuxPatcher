# Settlers 4 Widescreen Tool - Linux Edition

A Linux port of the Settlers 4 Widescreen Tool, written in Rust with an Iced GUI. This tool patches **The Settlers 4 - Gold Edition (GOG version v2.50.1508)** to support modern widescreen resolutions up to 1920×1200.

![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)
![Platform](https://img.shields.io/badge/platform-Linux-lightgrey.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

## Features

- **GOG Version Validation**: Only patches validated GOG Gold Edition v2.50.1508 installations
- **8 Widescreen Resolutions**:
  - 1024×600 (17:10)
  - 1280×720 (16:9)
  - 1280×800 (16:10)
  - 1366×768 (16:9)
  - 1440×900 (16:10)
  - 1680×1050 (16:10)
  - 1920×1080 (16:9)
  - 1920×1200 (16:10)
- **Auto-Detection**: Automatically scans `~/Games` for Settlers 4 installation
- **Restore to Default**: One-click revert to vanilla 1024×768 resolution using bundled defaults
- **Modern GUI**: Built with Iced for a native Linux experience
- **Safe Patching**: Validates GOG version before patching, prevents patching while game is running

## Installation

### Prerequisites

- Rust 1.70 or higher
- The Settlers 4 - Gold Edition (GOG version v2.50.1508)
- Linux (tested on Arch, should work on most distributions)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/Settlers4-Widescreen-Tool.git
cd Settlers4-Widescreen-Tool

# Build in release mode
cargo build --release

# The binary will be at: target/release/settlers4-widescreen-tool
```

### Installing

```bash
# Copy binary to local bin directory
sudo cp target/release/settlers4-widescreen-tool /usr/local/bin/

# Or run directly
./target/release/settlers4-widescreen-tool
```

## Usage

Run the application (GUI):

```bash
settlers4-widescreen-tool
```

The GUI will:
1. Auto-detect your Settlers 4 installation in `~/Games`
2. Validate it's a GOG version
3. Allow you to select target resolution
4. Patch the game (restoration to defaults is always available)

## How It Works

The tool performs two main operations:

1. **Replaces GfxEngine.dll**: Swaps the game's graphics engine DLL with a pre-patched version for the selected resolution
2. **Updates GameSettings.cfg**: Modifies the INI configuration file with new resolution settings

### File Structure

```
<Game Install Dir>/
├── S4.exe                       # Main game executable
├── Config/
│   └── GameSettings.cfg         # INI file with resolution settings
└── Exe/
    └── GfxEngine.dll            # Graphics engine (patched)
```

### GOG Version Detection

The tool validates installations by computing SHA1 hashes of `GfxEngine.dll` and comparing against known GOG Gold Edition v2.50.1508 hashes. Only valid GOG versions are allowed to be patched for safety.

## Safety Features

- **Version Validation**: Only patches validated GOG versions
- **Process Check**: Prevents patching while S4.exe is running
- **Restore Functionality**: Built-in restore to default files and resolution
- **Error Handling**: Clear error messages for all operations

## Troubleshooting

### "Invalid GOG version" Error

- **Cause**: Your GfxEngine.dll doesn't match known GOG v2.50.1508 hashes
- **Solution**: Ensure you have the GOG Gold Edition v2.50.1508. Other versions (Steam, CD, etc.) are not supported.

### Game Won't Start After Patching

- **Solution**: Use the "Restore to Default" action to re-apply the bundled default files (1024×768)

### Auto-Detection Not Finding Game

- **Solution**: Manually enter the full path to your Settlers 4 installation directory (the folder containing S4.exe)

### Game Running Error

- **Cause**: S4.exe process is still running
- **Solution**: Close the game completely before patching

## Known Limitations

- Only supports GOG Gold Edition v2.50.1508
- Game must be a Windows version (running via Wine/Proton on Linux)
- Auto-detection scans `~/Games` only (max 5 levels deep)
- Display resolution auto-detection falls back to 1920×1080 if detection fails

## Original Project

This is a Linux port of the original [Settlers4-Widescreen-Tool](https://github.com/FireEmerald/Settlers4-Widescreen-Tool) by FireEmerald, which was written for Windows in Visual Basic .NET.

### Key Differences

- **Platform**: Linux (vs Windows)
- **Language**: Rust (vs VB.NET)
- **GUI**: Iced (vs Windows Forms)
- **Additional Features**:
  - Restore to default button
  - Modern dark theme

## Technical Details

### Architecture

- **Language**: Rust (2021 edition)
- **GUI Framework**: Iced 0.13
- **Dependencies**:
  - `rust-ini` - INI file handling
  - `sha1` - Hash verification
  - `anyhow`/`thiserror` - Error handling

### Modules

- `resolution.rs` - Resolution definitions with embedded DLLs (2.7MB)
- `validation.rs` - GOG version SHA1 validation
- `patcher.rs` - Core patching logic
- `game_detection.rs` - Auto-detection of game directory
- `ini_handler.rs` - GameSettings.cfg reader/writer
- `gui.rs` - Iced GUI implementation
- `display.rs` - Display resolution detection

## Development

### Running Tests

```bash
cargo test
```

### Building Debug Version

```bash
cargo build
./target/debug/settlers4-widescreen-tool --cli
```

### Code Structure

```
src/
├── main.rs           # Entry point launching the GUI
├── gui.rs            # Iced GUI implementation
├── resolution.rs     # Resolution data with embedded DLLs
├── validation.rs     # GOG version validation
├── patcher.rs        # Core patching logic
├── game_detection.rs # Game directory scanner
├── ini_handler.rs    # INI file reader/writer
└── display.rs        # Display resolution detection

dlls/                 # Pre-patched DLL files (embedded at compile time)
├── GfxEngine_1024x600.dll
├── GfxEngine_1280x720.dll
├── ... (9 total DLLs)
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

GPL-3.0 License - See [LICENSE](LICENSE) file for details.

This project maintains the same license as the [original Windows tool](https://github.com/FireEmerald/Settlers4-Widescreen-Tool).

## Credits

- **Original Tool**: [FireEmerald](https://github.com/FireEmerald) - Settlers4-Widescreen-Tool (Windows version)
- **Linux Port**: Rouven - Rust/Iced implementation
- **Pre-patched DLLs**: From original tool by FireEmerald

## Disclaimer

This tool modifies game files. While it includes safety features like validation and a bundled default restore, use at your own risk. Always keep backups of your game installation.

This tool is not affiliated with or endorsed by Blue Byte, Ubisoft, or GOG.

## Support

For issues, questions, or contributions, please use the GitHub issue tracker.

---

**Enjoy widescreen gaming on Linux!** 🎮🐧
