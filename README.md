# STT Clippy 🎤✂️

**Speech-to-Text with Smart Clipboard Management**

STT Clippy is a desktop application that allows you to activate speech-to-text from anywhere on your desktop via a global hotkey. The transcribed text is then either pasted at the cursor position and/or saved to your clipboard. The application also supports a sophisticated clipboard management system with a buffer of multiple recent clips.

## ✨ Features

- **Global Hotkey Activation**: Trigger STT from any application with customizable hotkeys
- **Speech-to-Text**: Real-time transcription with local and cloud options
- **Smart Output**: Paste at cursor and/or copy to clipboard
- **Clipboard History**: Multi-clipboard buffer with search and quick access
- **Privacy First**: Local processing by default, no data sent to cloud unless explicitly enabled
- **Cross-Platform**: Support for Linux, macOS, and Windows
- **High Performance**: Optimized for low latency and minimal resource usage

## 🚀 Quick Start

### Prerequisites

- **Linux**: Ubuntu 20.04+, Debian 11+, Fedora 35+, or Arch Linux
- **macOS**: macOS 11.0+ (Big Sur)
- **Windows**: Windows 10 1903+ or Windows 11
- **Hardware**: 4GB RAM minimum, 8GB recommended

### Installation

Note: Prebuilt installers are not published yet. Build and run from source for now.

#### Build from source (all platforms)
```bash
# Prerequisites: Rust toolchain (https://rustup.rs/)
git clone <this repo URL>
cd clipstty
cargo build
```

### First Run

1. **Launch** the application
2. **Grant Permissions**:
   - Microphone access for audio capture
   - Accessibility permissions for paste injection (platform-dependent)
   - Clipboard access for clipboard management
3. **Configure Hotkeys**: Default is `Ctrl+Alt+S` for STT activation
4. **Test**: Press your hotkey and speak - the transcribed text should appear in your clipboard!

### Local model setup (optional for local STT)

The local backend uses a Whisper model file. By default, it looks for `ggml-large-v3-turbo-q8_0.bin` in the current directory. You can override this with the `WHISPER_MODEL_PATH` environment variable.

```bash
# Option 1: Use the default model (place in current directory)
# Download the default large-v3-turbo model
curl -L -o ggml-large-v3-turbo-q8_0.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q8_0.bin?download=true

# Option 2: Use a custom model with environment variable
# Example: download the base English model (adjust path as desired)
mkdir -p ~/models/whisper
curl -L -o ~/models/whisper/ggml-base.en.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin?download=true

# Point the app to your custom model
export WHISPER_MODEL_PATH=~/models/whisper/ggml-base.en.bin

# Optional: tune performance
export WHISPER_THREADS=$(getconf _NPROCESSORS_ONLN 2>/dev/null || sysctl -n hw.ncpu)
# macOS: enable Metal GPU acceleration (default on macOS)
export WHISPER_USE_GPU=1
```

### Run the CLI prototype (copies transcripts to clipboard)

```bash
# With default model (ggml-large-v3-turbo-q8_0.bin in current directory)
cargo run --bin stt_to_clipboard

# Or with custom model
export WHISPER_MODEL_PATH=~/models/whisper/ggml-base.en.bin
cargo run --bin stt_to_clipboard
```

This continuously listens for speech, transcribes detected segments, and copies the result to your clipboard.

### Run the main app (hotkey-activated, creates a config file on first run)

```bash
# With default model (ggml-large-v3-turbo-q8_0.bin in current directory)
cargo run --bin stt-clippy --features local-stt

# Or with custom model
export WHISPER_MODEL_PATH=~/models/whisper/ggml-base.en.bin
cargo run --bin stt-clippy --features local-stt
```

On first run, a default `stt-clippy.toml` will be created in your user config directory. Edit it to adjust audio, hotkeys, and output behavior.

### Environment variables

- `WHISPER_MODEL_PATH` (optional): path to a Whisper model `.bin` file (default: `ggml-large-v3-turbo-q8_0.bin`)
- `WHISPER_THREADS` (optional): number of CPU threads to use
- `WHISPER_USE_GPU` (optional, macOS default = on): `1` to enable GPU, `0` to force CPU

## 🎯 Usage

### Basic Speech-to-Text

1. **Press the hotkey** (`Ctrl+Alt+S` by default) from any application
2. **Speak clearly** - you'll see audio level indicators
3. **Wait for completion** - the app will automatically detect when you stop speaking
4. **Text appears** in your clipboard and/or pasted at the cursor

### Clipboard History

- **Open History**: Press `Ctrl+Alt+H` to open the clipboard history palette
- **Search**: Type to search through your clipboard history
- **Quick Access**: Use `Alt+1` through `Alt+9` for recent clips
- **Manage**: Pin important clips, delete old ones, or export your history

### Advanced Features

- **Push-to-Talk**: Hold the hotkey while speaking
- **Toggle Mode**: Press once to start, press again to stop
- **Language Detection**: Automatic language detection or manual selection
- **Model Selection**: Choose between different STT model sizes for speed vs. accuracy

## ⚙️ Configuration

### Audio Settings

```toml
[audio]
sample_rate = 16000          # Audio sample rate
vad_sensitivity = 0.5        # Voice activity detection sensitivity
vad_timeout = 2000           # Timeout in milliseconds
noise_reduction = true        # Enable noise reduction
```

### STT Settings

```toml
[stt]
backend = "local"            # local, cloud, or hybrid
model_size = "base"          # tiny, base, small, medium, large
language = "auto"            # Language or "auto" for detection
enable_punctuation = true    # Add punctuation automatically
```

### Hotkey Settings

```toml
[hotkeys]
primary = "Ctrl+Alt+S"       # Main STT activation
history_palette = "Ctrl+Alt+H" # Open clipboard history
quick_access = ["Alt+1", "Alt+2", "Alt+3"] # Quick clip access
```

### Privacy Settings

```toml
[privacy]
data_retention = "30d"       # How long to keep clipboard history
auto_expiry = true           # Automatically expire old clips
sensitive_apps = []          # Apps to exclude from clipboard access
encrypt_storage = true       # Encrypt clipboard history at rest
```

## 🏗️ Development

### Project Structure

```
stt-clippy/
├── src/                     # Source code
│   ├── core/               # Core application logic
│   ├── services/           # Service implementations
│   ├── platform/           # Platform-specific code
│   └── ui/                 # User interface
├── docs/                   # Documentation
├── tests/                  # Test suite
├── scripts/                # Build and deployment scripts
└── resources/              # Application resources
```

### Building from Source

#### Prerequisites

- **Rust**: 1.70+ (install from [rustup.rs](https://rustup.rs/))
- **Node.js**: 18+ (for UI development)
- **Platform Dependencies**:
  - **Linux**: `libpulse-dev`, `libasound2-dev`, `libx11-dev`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio Build Tools

#### Build Commands

```bash
# Clone the repository
git clone https://github.com/your-org/stt-clippy.git
cd stt-clippy

# Install dependencies
cargo build --release

# Run tests
cargo test

# Build for specific platform
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-pc-windows-msvc
```

### Development Workflow

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Make** your changes and add tests
4. **Run** the test suite: `cargo test`
5. **Commit** your changes: `git commit -m 'Add amazing feature'`
6. **Push** to the branch: `git push origin feature/amazing-feature`
7. **Open** a Pull Request

### Testing

```bash
# Run all tests
cargo test

# Run tests with coverage
cargo tarpaulin

# Run integration tests
cargo test --test integration

# Run performance benchmarks
cargo bench
```

## 📚 Documentation

- **[Development Roadmap](DEVELOPMENT_ROADMAP.md)**: Detailed development phases and milestones
- **[Architecture](ARCHITECTURE.md)**: System architecture and component design
- **[Technical Requirements](TECHNICAL_REQUIREMENTS.md)**: Detailed technical specifications
- **[API Reference](docs/api.md)**: Developer API documentation
- **[Contributing Guide](CONTRIBUTING.md)**: How to contribute to the project

## 🤝 Contributing

We welcome contributions from the community! Here are some ways you can help:

- **🐛 Report Bugs**: Use the [issue tracker](https://github.com/your-org/stt-clippy/issues)
- **💡 Suggest Features**: Open a feature request issue
- **📝 Improve Documentation**: Help make the docs better
- **🔧 Fix Issues**: Pick up an issue and submit a PR
- **🧪 Add Tests**: Help improve test coverage
- **🌍 Localization**: Help translate the application

### Contribution Guidelines

- Follow the existing code style and conventions
- Add tests for new functionality
- Update documentation as needed
- Ensure all tests pass before submitting
- Use conventional commit messages

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **OpenAI Whisper**: Speech recognition models
- **Faster-Whisper**: Efficient STT implementation
- **Silero VAD**: Voice activity detection
- **Tauri**: Cross-platform desktop framework
- **Community Contributors**: Everyone who helps improve STT Clippy

## 📞 Support

- **GitHub Issues**: [Bug reports and feature requests](https://github.com/your-org/stt-clippy/issues)
- **Discussions**: [Community discussions](https://github.com/your-org/stt-clippy/discussions)
- **Wiki**: [User guides and troubleshooting](https://github.com/your-org/stt-clippy/wiki)
- **Email**: support@stt-clippy.com

## 🔮 Roadmap

See our [Development Roadmap](DEVELOPMENT_ROADMAP.md) for detailed information about upcoming features and development phases.

### Upcoming Features

- **Phase 1**: Basic STT functionality and clipboard integration
- **Phase 2**: Paste at cursor capability
- **Phase 3**: Advanced clipboard history management
- **Phase 4**: Cross-platform optimization
- **Phase 5**: Advanced features and plugins

---

**Made with ❤️ by the STT Clippy Team**

*Transform your voice into text, anywhere, anytime.*
