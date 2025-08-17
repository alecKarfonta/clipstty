# Enhanced Audio Session Outputs

This document describes the enhanced audio session management functionality that creates comprehensive outputs when recording sessions end.

## Overview

When an audio recording session is stopped, the system now automatically generates multiple output files and formats to provide maximum utility and flexibility for the recorded content.

## Generated Outputs

### 1. Raw Audio File (`raw_audio.wav`)
- **Purpose**: Complete, unmodified recording of the entire session
- **Format**: WAV format with original quality settings
- **Content**: All audio captured during the session, including speech and silence
- **Use Case**: Full archival, backup, or when complete audio context is needed

### 2. Cleaned Audio File (`cleaned_audio.wav`)
- **Purpose**: Audio with silence periods removed for efficient listening
- **Format**: WAV format matching original quality
- **Content**: Only speech segments concatenated together
- **Processing**: Uses energy-based Voice Activity Detection (VAD) to identify and remove silence
- **Use Case**: Quick review, sharing, or when storage space is a concern

### 3. Individual Segment Files (`segments/` directory)
- **Purpose**: Each speech segment as a separate audio file
- **Format**: WAV files named `segment_001_<uuid>.wav`, `segment_002_<uuid>.wav`, etc.
- **Content**: Individual speech segments with precise timing
- **Metadata**: Each segment includes timing, transcript text (if available), and confidence scores
- **Use Case**: Detailed analysis, segment-specific sharing, or training data preparation

### 4. Comprehensive Metadata File (`session_metadata.json`)
- **Purpose**: Complete session information and segment details
- **Format**: JSON with structured metadata
- **Content**: Session info, timing data, transcript segments, file paths, and statistics
- **Use Case**: Programmatic access, analysis, or integration with other tools

## Directory Structure

After a session ends, the following directory structure is created:

```
sessions/
└── YYYY/MM/DD/
    └── <session_name>_<session_id>/
        ├── raw_audio.wav              # Complete recording
        ├── cleaned_audio.wav          # Silence removed
        ├── session_metadata.json      # Comprehensive metadata
        └── segments/                  # Individual segments
            ├── segment_001_<uuid>.wav
            ├── segment_002_<uuid>.wav
            └── segment_003_<uuid>.wav
```

## Metadata Structure

The `session_metadata.json` file contains:

```json
{
  "session": {
    "id": "<uuid>",
    "name": "Session Name",
    "description": "Optional description",
    "start_time": "2024-01-01T12:00:00Z",
    "end_time": "2024-01-01T12:05:00Z",
    "duration": {"secs": 300, "nanos": 0},
    "audio_source": "Microphone",
    "file_path": "/path/to/raw_audio.wav",
    "file_size": 12345678,
    "format_info": {
      "sample_rate": 44100,
      "channels": 1,
      "bit_depth": 16,
      "format": "WAV"
    },
    "transcript_segments": [...],
    "tags": ["meeting", "important"],
    "metadata": {},
    "state": "Stopped",
    "quality_metrics": {...}
  },
  "outputs": {
    "raw_audio_path": "/path/to/raw_audio.wav",
    "cleaned_audio_path": "/path/to/cleaned_audio.wav",
    "segments_directory": "/path/to/segments/",
    "metadata_path": "/path/to/session_metadata.json",
    "segments": [
      {
        "id": "<uuid>",
        "start_sample": 0,
        "end_sample": 88200,
        "start_time": {"secs": 0, "nanos": 0},
        "end_time": {"secs": 2, "nanos": 0},
        "duration": {"secs": 2, "nanos": 0},
        "text": "Transcribed text if available",
        "confidence": 0.95,
        "file_path": "/path/to/segment_001.wav",
        "file_size": 352800,
        "is_speech": true,
        "average_energy": 0.123
      }
    ],
    "total_raw_duration": {"secs": 300, "nanos": 0},
    "total_cleaned_duration": {"secs": 180, "nanos": 0},
    "silence_removed_duration": {"secs": 120, "nanos": 0}
  },
  "generated_at": "2024-01-01T12:05:01Z",
  "version": "0.1.0"
}
```

## Voice Activity Detection (VAD)

The system uses energy-based VAD to detect speech segments:

- **Frame Size**: 25ms windows for analysis
- **Hop Size**: 10ms overlap for smooth detection
- **Energy Threshold**: Configurable (default: 0.001)
- **Minimum Speech Duration**: 500ms to avoid noise
- **Maximum Silence Gap**: 300ms before ending a segment

## Configuration

Session output behavior can be configured through `SessionConfig`:

```rust
SessionConfig {
    auto_transcribe: true,           // Enable transcript generation
    real_time_transcription: true,   // Process during recording
    save_raw_audio: true,           // Save complete recording
    compress_audio: false,          // Enable audio compression
    quality_monitoring: true,       // Track audio quality metrics
    // ... other settings
}
```

## Testing

A test binary is provided to demonstrate the functionality:

```bash
cargo run --bin test_enhanced_session_manager
```

This test:
1. Creates a session with synthetic audio (speech + silence)
2. Adds transcript segments with timing
3. Stops the session to trigger output generation
4. Displays the generated files and metadata

## Benefits

1. **Flexibility**: Multiple formats for different use cases
2. **Efficiency**: Cleaned audio saves storage and listening time
3. **Precision**: Individual segments enable detailed analysis
4. **Metadata**: Rich information for programmatic processing
5. **Archival**: Complete raw audio preserves original context
6. **Integration**: JSON metadata enables easy tool integration

## Future Enhancements

Potential improvements include:
- Advanced VAD algorithms (neural network-based)
- Audio compression options
- Multiple output formats (MP3, FLAC, etc.)
- Speaker diarization for multi-speaker segments
- Automatic chapter/topic detection
- Cloud storage integration
- Real-time segment streaming
