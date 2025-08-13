# STT Clippy - Architecture Document

## System Overview

STT Clippy is designed as a modular, cross-platform desktop application with a focus on performance, reliability, and user privacy. The system architecture follows a layered approach with clear separation of concerns between platform-specific implementations and cross-platform core functionality.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    User Interface Layer                      │
├─────────────────────────────────────────────────────────────┤
│                  Application Core Layer                      │
├─────────────────────────────────────────────────────────────┤
│                   Service Layer                             │
├─────────────────────────────────────────────────────────────┤
│                  Platform Abstraction Layer                  │
├─────────────────────────────────────────────────────────────┤
│                   Platform Implementation                    │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. User Interface Layer
- **System Tray**: Minimal interface for status and quick access
- **Settings Panel**: Configuration management interface
- **History Palette**: Overlay interface for clipboard history
- **Status Notifications**: User feedback and error reporting

### 2. Application Core Layer
- **Event Bus**: Centralized event handling and routing
- **Configuration Manager**: Settings persistence and validation
- **Plugin System**: Extensible architecture for custom functionality
- **State Management**: Application state and lifecycle management

### 3. Service Layer
- **Audio Service**: Audio capture and processing
- **STT Service**: Speech-to-text processing engine
- **Clipboard Service**: Clipboard management and history
- **Hotkey Service**: Global hotkey registration and handling
- **Paste Service**: Text injection and paste simulation

### 4. Platform Abstraction Layer
- **Audio Interface**: Cross-platform audio capture abstraction
- **Clipboard Interface**: Platform-agnostic clipboard operations
- **Input Interface**: Cross-platform input simulation
- **System Interface**: Platform-specific system operations

### 5. Platform Implementation
- **Linux**: X11 and Wayland support
- **macOS**: Native macOS APIs and accessibility
- **Windows**: Windows-specific APIs and services

## Detailed Component Design

### Audio Service

The Audio Service handles all audio-related operations including capture, preprocessing, and streaming to the STT engine.

```rust
pub trait AudioCapture {
    fn start_capture(&mut self, config: AudioConfig) -> Result<(), AudioError>;
    fn stop_capture(&mut self) -> Result<(), AudioError>;
    fn get_audio_stream(&self) -> Result<AudioStream, AudioError>;
    fn is_capturing(&self) -> bool;
}

pub struct AudioService {
    capture: Box<dyn AudioCapture>,
    vad: VoiceActivityDetector,
    preprocessor: AudioPreprocessor,
    config: AudioConfig,
}
```

**Key Features:**
- Real-time audio streaming
- Voice Activity Detection (VAD)
- Audio preprocessing (noise reduction, normalization)
- Configurable sample rates and formats
- Platform-specific audio backend selection

### STT Service

The STT Service manages speech-to-text processing with support for multiple backends and streaming results.

```rust
pub trait STTBackend {
    fn transcribe_stream(&mut self, audio: &[f32]) -> Result<PartialResult, STTError>;
    fn finalize(&mut self) -> Result<FinalResult, STTError>;
    fn reset(&mut self) -> Result<(), STTError>;
}

pub struct STTService {
    backend: Box<dyn STTBackend>,
    processor: TextProcessor,
    config: STTConfig,
}
```

**Supported Backends:**
- **Local**: Faster-Whisper (CTranslate2)
- **Cloud**: OpenAI Whisper API, Deepgram, Google Speech-to-Text
- **Hybrid**: Local fallback with cloud enhancement

### Clipboard Service

The Clipboard Service manages clipboard operations and maintains a history buffer with metadata.

```rust
pub struct ClipboardService {
    history: ClipboardHistory,
    storage: ClipboardStorage,
    config: ClipboardConfig,
}

pub struct ClipboardHistory {
    clips: VecDeque<ClipboardItem>,
    max_items: usize,
    pinned_items: HashSet<String>,
}
```

**Features:**
- Persistent storage with SQLite
- Metadata tracking (timestamp, source, tags)
- Search and filtering capabilities
- Export/import functionality
- Privacy controls and data redaction

### Hotkey Service

The Hotkey Service manages global hotkey registration and conflict resolution across platforms.

```rust
pub trait HotkeyManager {
    fn register_hotkey(&mut self, key: Hotkey) -> Result<(), HotkeyError>;
    fn unregister_hotkey(&mut self, key: Hotkey) -> Result<(), HotkeyError>;
    fn is_registered(&self, key: Hotkey) -> bool;
}

pub struct HotkeyService {
    manager: Box<dyn HotkeyManager>,
    handlers: HashMap<Hotkey, Box<dyn HotkeyHandler>>,
    config: HotkeyConfig,
}
```

**Platform Support:**
- **Linux**: X11 (XGrabKey), Wayland (libinput)
- **macOS**: Carbon Hot Keys, Global Monitor
- **Windows**: RegisterHotKey, SetWindowsHookEx

### Paste Service

The Paste Service handles text injection and paste simulation with platform-specific implementations.

```rust
pub trait PasteInjector {
    fn inject_text(&self, text: &str) -> Result<(), PasteError>;
    fn can_inject(&self) -> bool;
    fn get_fallback_method(&self) -> FallbackMethod;
}

pub struct PasteService {
    injector: Box<dyn PasteInjector>,
    fallback: FallbackHandler,
    config: PasteConfig,
}
```

**Injection Methods:**
- **Linux X11**: XTest, XSendEvent
- **Linux Wayland**: wtype, ydotool (with fallback)
- **macOS**: Accessibility API, AppleScript
- **Windows**: SendInput, PostMessage

## Data Flow

### 1. Hotkey Activation Flow
```
User presses hotkey → HotkeyService → EventBus → AudioService.start_capture()
```

### 2. Audio Processing Flow
```
AudioService → VAD → STTService → TextProcessor → EventBus
```

### 3. Output Flow
```
EventBus → ClipboardService → PasteService → User's application
```

## Configuration Management

The application uses a hierarchical configuration system with environment-specific overrides:

```toml
[audio]
sample_rate = 16000
channels = 1
vad_sensitivity = 0.5
vad_timeout = 2000

[stt]
backend = "local"
model_size = "base"
language = "auto"
enable_punctuation = true

[clipboard]
max_history = 500
auto_save = true
privacy_mode = "standard"

[hotkeys]
primary = "Ctrl+Alt+S"
history_palette = "Ctrl+Alt+H"
quick_access = ["Alt+1", "Alt+2", "Alt+3"]

[paste]
mode = "both"
fallback = "clipboard"
restore_clipboard = true
```

## Security and Privacy

### Data Handling
- **Local Processing**: All audio processing occurs locally by default
- **Encrypted Storage**: Clipboard history encrypted at rest
- **No Network Access**: Local mode requires no internet connection
- **Data Retention**: Configurable auto-expiry policies

### Permissions
- **Microphone Access**: Required for audio capture
- **Accessibility**: Required for paste injection (platform-dependent)
- **Clipboard Access**: Required for clipboard management
- **System Integration**: Required for global hotkeys

### Privacy Controls
- **App Exclusion**: Block sensitive applications
- **Data Redaction**: Automatic content filtering
- **Export Controls**: User-controlled data export
- **Audit Logging**: Track data access and modifications

## Performance Considerations

### Latency Optimization
- **Streaming Processing**: Real-time audio streaming to STT
- **Model Preloading**: Warm STT models on startup
- **Background Processing**: Non-blocking UI operations
- **Memory Management**: Efficient buffer management

### Resource Management
- **CPU Usage**: Configurable processing limits
- **Memory Usage**: Bounded memory consumption
- **GPU Acceleration**: Optional GPU acceleration for STT
- **Power Management**: Low-power mode for mobile devices

## Error Handling and Recovery

### Error Categories
- **Audio Errors**: Capture failures, device issues
- **STT Errors**: Model loading, processing failures
- **System Errors**: Permission issues, platform limitations
- **User Errors**: Configuration problems, invalid inputs

### Recovery Strategies
- **Graceful Degradation**: Fallback to alternative methods
- **Automatic Retry**: Retry failed operations with backoff
- **User Notification**: Clear error messages with solutions
- **Logging and Debugging**: Comprehensive error logging

## Testing Strategy

### Unit Testing
- **Component Isolation**: Test individual components in isolation
- **Mock Dependencies**: Use mock objects for external dependencies
- **Edge Case Coverage**: Test boundary conditions and error cases

### Integration Testing
- **End-to-End Workflows**: Test complete user workflows
- **Cross-Platform Testing**: Verify behavior across platforms
- **Performance Testing**: Measure latency and resource usage

### User Acceptance Testing
- **Real-World Scenarios**: Test with actual user workflows
- **Accessibility Testing**: Verify accessibility compliance
- **Usability Testing**: Evaluate user experience and learnability

## Deployment and Distribution

### Build System
- **Cross-Platform Builds**: Automated builds for all target platforms
- **Dependency Management**: Automated dependency resolution and updates
- **Code Signing**: Digital signatures for security and trust
- **Release Automation**: Automated release and distribution

### Package Distribution
- **Linux**: AppImage, DEB, RPM packages
- **macOS**: DMG installer, App Store distribution
- **Windows**: MSIX, NSIS installer
- **Updates**: Automatic update system with rollback capability

## Monitoring and Observability

### Metrics Collection
- **Performance Metrics**: Latency, throughput, resource usage
- **Error Rates**: Failure rates and error patterns
- **Usage Statistics**: Feature usage and user behavior
- **System Health**: Application stability and reliability

### Logging
- **Structured Logging**: JSON-formatted logs for analysis
- **Log Levels**: Configurable logging verbosity
- **Log Rotation**: Automatic log file management
- **Remote Logging**: Optional remote log aggregation

### Health Checks
- **Service Health**: Monitor service availability and performance
- **Dependency Health**: Check external service dependencies
- **Resource Health**: Monitor system resource usage
- **User Experience**: Track user-facing metrics and issues

## Future Extensibility

### Plugin Architecture
- **Custom STT Backends**: User-defined STT engines
- **Text Processing**: Custom text post-processing plugins
- **Output Handlers**: Custom output destinations and formats
- **Integration Hooks**: Third-party application integrations

### API and SDK
- **REST API**: HTTP API for external integrations
- **Native SDK**: Language-specific SDKs for developers
- **Webhook Support**: Event-driven integration capabilities
- **Custom Protocols**: Support for custom communication protocols

This architecture provides a solid foundation for building a robust, scalable, and maintainable STT Clippy application while ensuring cross-platform compatibility and user privacy.
