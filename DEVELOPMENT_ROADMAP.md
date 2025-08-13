# STT Clippy - Development Roadmap

## Project Overview
STT Clippy is a desktop application that allows users to activate speech-to-text from anywhere on their desktop via a global hotkey. The transcribed text is then either pasted at the cursor position and/or saved to the user's clipboard. The application also supports a sophisticated clipboard management system with a buffer of multiple recent clips.

## Core Features
- **Global Hotkey Activation**: Trigger STT from any application
- **Speech-to-Text**: Real-time transcription with local and cloud options
- **Smart Output**: Paste at cursor and/or copy to clipboard
- **Clipboard History**: Multi-clipboard buffer with search and quick access
- **Cross-Platform**: Support for Linux, macOS, and Windows

## Technical Stack
- **Core App**: Tauri (Rust + minimal web UI) or Electron (Node + Rust/native addon)
- **STT Engine**: Faster-Whisper (CTranslate2) for local processing
- **VAD**: Silero VAD or WebRTC VAD for voice activity detection
- **Global Hotkeys**: Platform-native hooks
- **Clipboard**: Platform-native APIs with local database storage
- **Paste Injection**: Platform-specific methods (X11, Wayland, macOS, Windows)

---

## Phase 0 — Decision and Feasibility Spike
**Duration**: 1 week  
**Goal**: Validate technical approach and choose implementation stack

### Checklist
- [ ] Choose framework: Tauri vs Electron
  - [ ] Evaluate distribution complexity
  - [ ] Assess footprint and performance
  - [ ] Review OS integration capabilities
- [ ] Select default STT backend
  - [ ] Test Faster-Whisper performance on target hardware
  - [ ] Define cloud abstraction interface
  - [ ] Measure model loading times
- [ ] Validate audio pipeline
  - [ ] Test audio capture on target OS
  - [ ] Verify VAD performance
  - [ ] Test streaming partial results
- [ ] Measure latency benchmarks
  - [ ] Hotkey → first partial (<500ms target)
  - [ ] Full sentence completion (<2-3s target)
- [ ] Verify paste simulation feasibility
  - [ ] Test X11 injection methods
  - [ ] Evaluate Wayland limitations and fallbacks
  - [ ] Document macOS/Windows approaches
- [ ] Define permissions and privacy requirements
  - [ ] Microphone access only
  - [ ] No network access for local mode
  - [ ] Data retention policies

### Deliverables
- Technical decision document
- Performance benchmarks
- Risk assessment
- Architecture diagram

---

## Phase 1 — Project Scaffolding and Core Services
**Duration**: 2-3 weeks  
**Goal**: Establish project foundation and basic services

### Checklist
- [ ] Repository setup
  - [ ] Initialize git repository
  - [ ] Set up CI/CD pipeline
  - [ ] Configure linting and formatting
  - [ ] Plan code signing strategy
- [ ] Application bootstrap
  - [ ] System tray integration
  - [ ] Background service architecture
  - [ ] Settings persistence layer
- [ ] Audio capture module
  - [ ] Linux: PipeWire/PulseAudio support
  - [ ] Cross-platform audio abstractions
  - [ ] Sample rate and format handling
- [ ] Voice Activity Detection
  - [ ] Implement Silero VAD
  - [ ] Push-to-talk and toggle modes
  - [ ] Configurable sensitivity settings
- [ ] STT interface abstraction
  - [ ] Local processing adapter
  - [ ] Cloud service adapter
  - [ ] Streaming partial results
  - [ ] Final result processing
- [ ] Clipboard service
  - [ ] Read/write operations
  - [ ] Initial ring buffer (in-memory)
  - [ ] Platform abstraction layer
- [ ] Logging and monitoring
  - [ ] Structured logging framework
  - [ ] Rotating log files
  - [ ] Basic crash handling
  - [ ] Performance metrics collection

### Deliverables
- Working headless service
- Audio capture and VAD pipeline
- Basic STT integration
- Logging and monitoring system

---

Implement a method for selecting between different whisper models, or this new voxtral model
@https://huggingface.co/mistralai/Voxtral-Mini-3B-2507 

## Phase 2 — MVP: Hotkey → Transcribe → Clipboard
**Duration**: 2 weeks  
**Goal**: Basic end-to-end functionality

### Checklist
- [ ] Global hotkey system
  - [ ] Platform-specific hotkey registration
  - [ ] Configurable key combinations
  - [ ] Default: Ctrl+Alt+S
  - [ ] Conflict detection and resolution
- [ ] STT workflow integration
  - [ ] Hotkey triggers audio capture
  - [ ] VAD controls recording start/stop
  - [ ] STT processes audio stream
  - [ ] Final transcript generation
- [ ] Output handling
  - [ ] Copy to clipboard functionality
  - [ ] Basic error handling
  - [ ] Success/failure feedback
- [ ] User experience
  - [ ] Audible start/stop cues
  - [ ] Tray icon state indicators
  - [ ] Basic status notifications
- [ ] Configuration system
  - [ ] Toggle auto-copy to clipboard
  - [ ] Language selection
  - [ ] Model size configuration
  - [ ] VAD sensitivity tuning

### Acceptance Criteria
- [ ] Global hotkey reliably activates STT from any application
- [ ] Transcribed text appears in clipboard within latency targets
- [ ] No crashes during 30-minute continuous use
- [ ] Basic error handling and user feedback

---

## Phase 3 — Paste at Cursor (with Robust Fallbacks)
**Duration**: 2 weeks  
**Goal**: Direct text insertion with graceful degradation

### Checklist
- [ ] Platform-specific paste injection
  - [ ] Linux X11: XTest simulation
  - [ ] Linux Wayland: wtype/ydotool helpers
  - [ ] macOS: Accessibility API
  - [ ] Windows: SendInput API
- [ ] Fallback mechanisms
  - [ ] Clipboard-only mode
  - [ ] Manual paste instructions
  - [ ] Error notifications
- [ ] Action mode configuration
  - [ ] Clipboard only
  - [ ] Paste only
  - [ ] Both (clipboard + paste)
- [ ] Clipboard integrity
  - [ ] Restore previous clipboard after paste
  - [ ] Handle clipboard conflicts
  - [ ] Data validation

### Acceptance Criteria
- [ ] Reliable paste injection on X11
- [ ] Clear fallback behavior on Wayland
- [ ] Configurable action modes
- [ ] No data loss during operations

---

## Phase 4 — Clipboard History Buffer (Multi-Clipboard)
**Duration**: 2 weeks  
**Goal**: Sophisticated clipboard management

### Checklist
- [ ] Persistent storage
  - [ ] Replace in-memory buffer with SQLite
  - [ ] Configurable capacity (100-1000 items)
  - [ ] Data migration and backup
- [ ] Metadata management
  - [ ] Timestamp tracking
  - [ ] Source identification (STT/manual)
  - [ ] Tagging system
  - [ ] Pin/unpin functionality
  - [ ] App context tracking
- [ ] History palette interface
  - [ ] Hotkey-triggered overlay
  - [ ] Searchable clip list
  - [ ] Keyboard navigation
  - [ ] Quick selection (Enter to paste/copy)
- [ ] Quick access hotkeys
  - [ ] Alt+1..9 for recent clips
  - [ ] Configurable shortcuts
  - [ ] Custom key combinations
- [ ] Advanced operations
  - [ ] Pin/unpin clips
  - [ ] Delete individual clips
  - [ ] Clear all history
  - [ ] Export/import functionality
- [ ] Privacy features
  - [ ] Sensitive app redaction
  - [ ] Exclusion lists
  - [ ] Data retention policies

### Acceptance Criteria
- [ ] History persists across application restarts
- [ ] Palette opens in <120ms with 1000 items
- [ ] Search completes in <200ms
- [ ] Selection and paste work from any application

---

## Phase 5 — UX Polish and STT Quality
**Duration**: 2 weeks  
**Goal**: Enhanced user experience and transcription quality

### Checklist
- [ ] Text processing options
  - [ ] Punctuation insertion
  - [ ] Smart capitalization
  - [ ] Auto-newline insertion
  - [ ] Space handling
- [ ] Capture modes
  - [ ] Push-to-talk vs toggle
  - [ ] VAD timeout configuration
  - [ ] Silence trimming
  - [ ] Background noise reduction
- [ ] Language support
  - [ ] Auto-language detection
  - [ ] Fixed language selection
  - [ ] Dialect options
  - [ ] Accent handling
- [ ] Real-time feedback
  - [ ] Partial transcript preview
  - [ ] Confidence indicators
  - [ ] Progress visualization
- [ ] Error handling
  - [ ] Actionable error messages
  - [ ] Troubleshooting guides
  - [ ] Recovery suggestions

### Acceptance Criteria
- [ ] Configurable dictation styles
- [ ] Reduced manual editing requirements
- [ ] Clear error guidance
- [ ] Improved transcription accuracy

---

## Phase 6 — Packaging and Updates
**Duration**: 2 weeks  
**Goal**: Distribution and maintenance

### Checklist
- [ ] Build system
  - [ ] Multi-platform build pipelines
  - [ ] Automated testing
  - [ ] Code signing
  - [ ] Release automation
- [ ] Package distribution
  - [ ] Linux: AppImage, DEB, RPM
  - [ ] macOS: DMG, App Store
  - [ ] Windows: MSIX, NSIS installer
- [ ] Model management
  - [ ] Incremental downloads
  - [ ] Local caching
  - [ ] Model selection UI
  - [ ] Version management
- [ ] Update system
  - [ ] Auto-update channel
  - [ ] Release notes
  - [ ] Rollback capability
  - [ ] Update notifications
- [ ] First-run experience
  - [ ] Permission requests
  - [ ] Hotkey tutorial
  - [ ] Wayland helper prompts
  - [ ] Configuration wizard

### Acceptance Criteria
- [ ] Clean installation on fresh systems
- [ ] Reliable update process
- [ ] Clear first-run guidance
- [ ] Proper permission handling

---

## Phase 7 — Observability, Performance, and Power
**Duration**: 2-3 weeks  
**Goal**: Production readiness and optimization

### Checklist
- [ ] Monitoring and observability
  - [ ] Structured logging
  - [ ] Performance metrics
  - [ ] Error tracking
  - [ ] Usage analytics (opt-in)
- [ ] Performance optimization
  - [ ] Latency dashboards
  - [ ] Resource usage monitoring
  - [ ] Bottleneck identification
  - [ ] Optimization implementation
- [ ] Resource management
  - [ ] CPU/GPU usage caps
  - [ ] Low-power mode
  - [ ] Memory pressure handling
  - [ ] Model lifecycle management
- [ ] Model optimization
  - [ ] Quantized models (int8/int4)
  - [ ] Warm-up strategies
  - [ ] Preloading on idle
  - [ ] Dynamic model switching

### Acceptance Criteria
- [ ] Meets latency targets on baseline hardware
- [ ] Respects resource budgets
- [ ] Comprehensive monitoring
- [ ] Performance regression detection

---

## Phase 8 — Advanced Features (Optional)
**Duration**: 2-3 weeks  
**Goal**: Enhanced functionality and customization

### Checklist
- [ ] Command mode
  - [ ] Phrase-to-action mapping
  - [ ] "New line", "tab" commands
  - [ ] Safe command execution
  - [ ] Custom command definitions
- [ ] Templates and macros
  - [ ] Frequently used phrases
  - [ ] Hotkey mapping
  - [ ] Context-aware templates
  - [ ] Import/export functionality
- [ ] Post-processing plugins
  - [ ] Custom regex rules
  - [ ] Capitalization patterns
  - [ ] Formatting rules
  - [ ] Plugin architecture
- [ ] App-specific behavior
  - [ ] Context-aware routing
  - [ ] Different punctuation rules
  - [ ] App-specific templates
  - [ ] Behavior customization

### Acceptance Criteria
- [ ] Advanced features are optional
- [ ] No core functionality regressions
- [ ] Extensible architecture
- [ ] Clear feature documentation

---

## Phase 9 — Cross-Platform Expansion
**Duration**: 3-4 weeks  
**Goal**: Full platform support

### Checklist
- [ ] macOS implementation
  - [ ] NSPasteboard integration
  - [ ] Accessibility API for paste
  - [ ] Hardened runtime
  - [ ] Microphone entitlements
- [ ] Windows implementation
  - [ ] Global hotkey system
  - [ ] SendInput API
  - [ ] Clipboard APIs
  - [ ] Installer packaging
- [ ] Platform abstraction
  - [ ] Shared core functionality
  - [ ] Minimal platform-specific code
  - [ ] Consistent behavior
  - [ ] Platform-specific optimizations

### Acceptance Criteria
- [ ] Feature parity across platforms
- [ ] Platform-specific quirks documented
- [ ] Consistent user experience
- [ ] Reliable functionality on all targets

---

## Phase 10 — Security, Privacy, and Compliance
**Duration**: 2-3 weeks  
**Goal**: Production security and compliance

### Checklist
- [ ] Security implementation
  - [ ] Local-only mode guarantee
  - [ ] Network access controls
  - [ ] Input validation
  - [ ] Secure storage
- [ ] Privacy protection
  - [ ] Encrypted local storage
  - [ ] Data redaction options
  - [ ] Auto-expiry policies
  - [ ] Privacy controls
- [ ] Compliance and documentation
  - [ ] Privacy policy
  - [ ] Data handling documentation
  - [ ] Threat model
  - [ ] Security audit
- [ ] User control
  - [ ] Clear permissions
  - [ ] Data export/delete
  - [ ] Opt-out mechanisms
  - [ ] Transparency tools

### Acceptance Criteria
- [ ] Privacy checklist compliance
- [ ] No sensitive data exposure
- [ ] Clear user controls
- [ ] Comprehensive documentation

---

## Risk Assessment and Mitigation

### High-Risk Items
1. **Wayland Input Injection Limitations**
   - **Risk**: Limited paste injection capabilities
   - **Mitigation**: Clipboard-first approach, helper workflow, clear documentation

2. **STT Latency on Low-End Hardware**
   - **Risk**: Poor performance on older systems
   - **Mitigation**: Quantized models, cloud fallback, performance monitoring

3. **Global Hotkey Conflicts**
   - **Risk**: System-level conflicts
   - **Mitigation**: Conflict detection, per-environment overrides, user configuration

4. **Accessibility and Sandboxing**
   - **Risk**: Permission and security restrictions
   - **Mitigation**: Clear permission documentation, self-tests, fallback modes

### Medium-Risk Items
1. **Cross-Platform Audio Handling**
   - **Risk**: Platform-specific audio quirks
   - **Mitigation**: Comprehensive testing, platform abstraction, fallback mechanisms

2. **Model Distribution and Updates**
   - **Risk**: Large model downloads and updates
   - **Mitigation**: Incremental updates, local caching, user control

3. **Clipboard Data Integrity**
   - **Risk**: Data corruption or loss
   - **Mitigation**: Validation, backup, error recovery

---

## Testing Strategy

### Test Matrix
- **Platforms**: Linux (X11/Wayland), macOS, Windows
- **Application Types**: Terminals, browsers, code editors, chat apps, office suites
- **Edge Cases**: Long dictations, rapid toggles, network loss, sleep/wake, device changes

### Testing Phases
1. **Unit Testing**: Core functionality, edge cases
2. **Integration Testing**: End-to-end workflows
3. **Performance Testing**: Latency, resource usage
4. **Compatibility Testing**: Platform-specific behavior
5. **User Acceptance Testing**: Real-world usage scenarios

### Automated Testing
- [ ] Unit test coverage >80%
- [ ] Integration test suite
- [ ] Performance regression tests
- [ ] Cross-platform build verification
- [ ] Security vulnerability scanning

---

## Success Metrics

### Performance Targets
- **Latency**: Hotkey → first partial <500ms, full sentence <2-3s
- **Reliability**: >95% success rate on supported environments
- **Resource Usage**: <100MB RAM, <5% CPU during idle
- **Startup Time**: <2 seconds from launch to ready

### User Experience Targets
- **Accuracy**: >95% transcription accuracy on clear speech
- **Usability**: <5 minutes to learn basic functionality
- **Accessibility**: Full keyboard navigation, screen reader support
- **Customization**: Configurable for 80% of use cases

### Quality Targets
- **Stability**: <1 crash per 100 hours of use
- **Updates**: <5% failure rate on automatic updates
- **Support**: <24 hour response time for critical issues
- **Documentation**: Comprehensive user and developer guides

---

## Timeline and Milestones

### Phase Timeline
- **Phases 0-2**: 2-3 weeks (MVP)
- **Phases 3-4**: 2 weeks (Core features)
- **Phases 5-6**: 2 weeks (Polish and distribution)
- **Phases 7-8**: 2-3 weeks (Optimization and advanced features)
- **Phases 9-10**: 3-4 weeks (Cross-platform and security)

### Key Milestones
1. **Week 3**: Basic STT functionality
2. **Week 5**: Paste at cursor capability
3. **Week 7**: Clipboard history system
4. **Week 9**: Production-ready application
5. **Week 12**: Cross-platform support
6. **Week 15**: Security and compliance complete

### Release Schedule
- **Alpha**: End of Phase 2 (Week 3)
- **Beta**: End of Phase 4 (Week 7)
- **Release Candidate**: End of Phase 6 (Week 9)
- **Production**: End of Phase 10 (Week 15)

---

## Resource Requirements

### Development Team
- **Lead Developer**: Full-time (15 weeks)
- **UI/UX Designer**: Part-time (8 weeks)
- **QA Engineer**: Part-time (10 weeks)
- **DevOps Engineer**: Part-time (6 weeks)

### Infrastructure
- **Development Environment**: Cross-platform testing VMs
- **CI/CD**: Automated build and test pipelines
- **Distribution**: Package hosting and update servers
- **Monitoring**: Application performance monitoring

### External Dependencies
- **STT Models**: Faster-Whisper, cloud APIs
- **Audio Libraries**: Platform-specific audio capture
- **UI Framework**: Tauri or Electron
- **Database**: SQLite for local storage

---

## Conclusion

This roadmap provides a structured approach to building STT Clippy, with clear phases, deliverables, and acceptance criteria. The phased approach allows for iterative development and early user feedback, while the comprehensive testing strategy ensures quality and reliability.

Key success factors include:
- Early validation of technical feasibility
- Focus on core functionality before advanced features
- Comprehensive testing across platforms and use cases
- Clear risk mitigation strategies
- Regular user feedback and iteration

The project is designed to deliver a production-ready application within 15 weeks, with the flexibility to adjust scope based on user feedback and technical challenges encountered during development.
