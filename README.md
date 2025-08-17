# STT Clippy 🎤✂️

Tired of typing instructions to your AI? 

**Transform your voice into text, anywhere, anytime.**

STT Clippy is a powerful desktop application that converts speech to text with a simple hotkey press. Whether you're writing emails, taking notes, or coding, just press a key, speak, and watch your words appear instantly.

## 🎬 Demo Videos

> **Coming Soon**: Video demonstrations of key features

| Feature | Demo | Description |
|---------|------|-------------|
| **STT to Clipboard** | 🎥 [Watch Demo](https://www.youtube.com/watch?v=E2_cI0sbckY) | Basic speech-to-text with clipboard integration |
| **Instant Output** | 🎥 [Watch Demo](https://www.youtube.com/watch?v=1ZcDpYB4nhQ) | Real-time text output as you speak |
| **Session Recording** | 🎥 [Watch Demo](https://www.youtube.com/watch?v=M8skn5w5y7c) | Advanced recording sessions with voice commands |

## ✨ Key Features

**Voice activcated detection** - Automatically transcribe as you speak with configuration throgh voice commands 
**Different modes** - Auto-copy to clipboard or paste directly where you're typing as you speak  
**Voice Commands** - Control the application and enable differnt modes through voice commands
**Session Recording** - Record named sessions of audio from a microphone or your system audio. Exports full audio, slience removed audio and broken out segments of speech. 

## 🚀 Quick Start

### 1. Install Rust
```bash
# Install Rust toolchain (if you don't have it)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Clone and Build
```bash
git clone https://github.com/alecKarfonta/clipstty.git
cd clipstty
cargo build --release
```

### 3. Get a Speech Model (Optional for Local STT)
```bash
# Download the default Whisper model (recommended)
curl -L -o ggml-large-v3-turbo-q8_0.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q8_0.bin?download=true
```

### 4. Run STT Clippy
```bash
# Basic STT to clipboard
cargo run --bin stt_to_clipboard

# Or run the full app with hotkeys
cargo run --bin stt-clippy --features local-stt
```

### 5. Test It Out
1. Press `Ctrl+Alt+S` (default hotkey)
2. Speak clearly into your microphone
3. Your text appears in the clipboard! 🎉

> **Need Help?** Check the [troubleshooting section](#-troubleshooting) below.

## 🎯 Usage Examples

### Basic Speech-to-Text
```bash
# Start listening and copy to clipboard
cargo run --bin stt_to_clipboard
```
Press `Ctrl+C` to stop. Speak clearly and your text will be copied to clipboard automatically.

### Voice Commands
```bash
# Test voice commands with real audio sessions
cargo run --bin test_recording_commands
```
Try saying:
- "Start recording session"
- "Stop recording"
- "Show status"
- "Enable debug mode"

### Advanced Configuration
```bash
# Use a different Whisper model
export WHISPER_MODEL_PATH=~/models/whisper/ggml-base.en.bin

# Optimize performance
export WHISPER_THREADS=8
export WHISPER_USE_GPU=1  # Enable GPU acceleration (macOS)

cargo run --bin stt_to_clipboard
```

## 🔧 Development & Testing

<details>
<summary><strong>Click to expand testing utilities</strong></summary>

### Voice Command Testing
```bash
# Test voice commands with real audio sessions
cargo run --bin test_recording_commands

# Test session manager functionality  
cargo run --bin test_enhanced_session_manager

# Integration testing for recording pipeline
cargo run --bin test_recording_integration
```

### Audio & TTS Testing
```bash
# Test basic TTS functionality
cargo run --bin test_tts_simple

# Advanced TTS debugging
cargo run --bin debug_tts

# Voice command test recorder
cargo run --bin test_recorder
```

### Debug Tools
```bash
# Debug audio recording with session management
cargo run --bin debug_audio_recording
```

</details>

## 📖 Documentation & Configuration

<details>
<summary><strong>Click to expand advanced configuration</strong></summary>

### Environment Variables
- `WHISPER_MODEL_PATH`: Path to Whisper model file (default: `ggml-large-v3-turbo-q8_0.bin`)
- `WHISPER_THREADS`: Number of CPU threads to use
- `WHISPER_USE_GPU`: Enable GPU acceleration (`1` for on, `0` for off)

### Configuration File
On first run, `stt-clippy.toml` is created in your user config directory:

```toml
[audio]
sample_rate = 16000
vad_sensitivity = 0.5
vad_timeout = 2000

[stt]
backend = "local"
model_size = "base"
language = "auto"

[hotkeys]
primary = "Ctrl+Alt+S"
history_palette = "Ctrl+Alt+H"
```

### Available Voice Commands (50+)
- **Recording**: "start recording", "stop recording", "pause recording"
- **STT**: "switch to whisper model", "set language to english"
- **System**: "show status", "enable debug mode", "restart service"
- **Transcripts**: "search transcripts", "export as text", "show recent"

</details>

## 🛠️ Troubleshooting

### Common Issues

**No audio detected?**
- Check microphone permissions
- Verify your default audio input device
- Try adjusting `vad_sensitivity` in config

**Model not found?**
- Download the Whisper model (see Quick Start step 3)
- Check `WHISPER_MODEL_PATH` environment variable

**Hotkey not working?**
- Grant accessibility permissions (macOS/Linux)
- Check for hotkey conflicts with other apps
- Try a different hotkey combination in config

**Performance issues?**
- Use a smaller model (`ggml-base.en.bin`)
- Adjust `WHISPER_THREADS` to match your CPU cores
- Enable GPU acceleration with `WHISPER_USE_GPU=1`

## 🤝 Contributing

We welcome contributions! Here's how to get started:

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Make** your changes and add tests
4. **Run** tests: `cargo test`
5. **Submit** a Pull Request

### Development Setup
```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/clipstty.git
cd clipstty

# Build and test
cargo build
cargo test
```

### Platform Dependencies
- **Linux**: `libpulse-dev`, `libasound2-dev`, `libx11-dev`
- **macOS**: Xcode Command Line Tools  
- **Windows**: Visual Studio Build Tools

## 📚 Documentation

- **[Enhanced Session Outputs](ENHANCED_SESSION_OUTPUTS.md)**: Voice command session management
- **[Test Recording Guide](TEST_RECORDING_GUIDE.md)**: Guide for testing voice commands
- **[Contributing Guide](CONTRIBUTING.md)**: How to contribute to the project

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **OpenAI Whisper**: Speech recognition models
- **Faster-Whisper**: Efficient STT implementation  
- **Silero VAD**: Voice activity detection
- **Community Contributors**: Everyone who helps improve STT Clippy

## 📞 Support

- **GitHub Issues**: [Bug reports and feature requests](https://github.com/alecKarfonta/clipstty/issues)
- **Discussions**: [Community discussions](https://github.com/alecKarfonta/clipstty/discussions)

## 🔮 Roadmap

### ✅ Completed (Phase 1)
- Enhanced voice command system with 50+ commands
- Real audio session integration  
- Advanced session management with metadata tracking
- Comprehensive testing and debugging tools

### 🚧 In Progress (Phase 2)
- LLM integration for "Ask Grok" style voice commands
- Advanced STT model switching
- Improved clipboard history management

### 📋 Planned (Phase 3+)
- Paste at cursor capability
- Cross-platform UI improvements
- Plugin system and automation features

---

**Made with ❤️ by the STT Clippy Team (Me and Claude)**

*Transform your voice into text, anywhere, anytime.*
