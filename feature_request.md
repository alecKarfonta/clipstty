-[ ] The saved output audio file is not right. It is massive 18 minute audio cilp and does not actually capture my command phrase in the audio


- [ ] in test script suppress whisper framework logging

- [ ] in test script do not unload model after transcriptions, keep it in memory

- [ ]  Make the TTS feedback cross compatible across operating systems. This will require looking into how to do TTS natively on Windows OS and Linux OS.


- [ ]  The speech detection is often cutting off the first word of my sentence, look into how we can use a buffer to prevent this.

- [ ] Use word stamps from the model output in order to trim audio recorderd by test_recorder to only be the audio around the specific phrase being tested

2025-08-16T18:08:24.124322Z  INFO stt_clippy::services::tts: TTS speaking and waiting: Recording saved successfully as phrase_001_enable_vad.wav
2025-08-16T18:08:28.776672Z  INFO stt_clippy::services::tts: TTS completed speaking: Recording saved successfully as phrase_001_enable_vad.wav
2025-08-16T18:08:28.776929Z  INFO stt_clippy::services::tts: TTS finished speaking: Recording saved successfully as phrase_001_enable_vad.wav
📝 Transcription: " test recording next. Start test recording next. and then. and then. start test recording next. and then. start test recording next. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then."
🎯 Expected: "enable vad"
📈 Similarity: 0.0%
2025-08-16T18:08:28.781966Z  INFO stt_clippy::services::tts: TTS speaking and waiting: Recording may need improvement. Expected 'enable vad', but got ' test recording next. Start test recording next. and then. and then. start test recording next. and then. start test recording next. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then.'
2025-08-16T18:08:49.278953Z  INFO stt_clippy::services::tts: TTS completed speaking: Recording may need improvement. Expected 'enable vad', but got ' test recording next. Start test recording next. and then. and then. start test recording next. and then. start test recording next. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then.'
2025-08-16T18:08:49.279083Z  INFO stt_clippy::services::tts: TTS finished speaking: Recording may need improvement. Expected 'enable vad', but got ' test recording next. Start test recording next. and then. and then. start test recording next. and then. start test recording next. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then.'
⚠️ Recording doesn't match well enough (similarity: 0.0%)
   Expected: "enable vad"
   Got: " test recording next. Start test recording next. and then. and then. start test recording next. and then. start test recording next. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then."
💡 Tip: Extra words were detected. Speak the exact phrase without adding extra words.
   Please try again. Say 'start test recording next' to re-record this phrase.
2025-08-16T18:08:49.279786Z  INFO stt_clippy::services::tts: TTS speaking and waiting: Expected 'enable vad', but heard ' test recording next. Start test recording next. and then. and then. start test recording next. and then. start test recording next. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then. and then.'. Extra words were detected. Speak the exact phrase without adding extra words.. Please try again.


- [ ] I said "set language to English" It heard" 

Transcription:  language to English.
2025-08-16T18:10:09.476091Z  INFO runner: [stt_to_clipboard].main transcribed len=21 text=" language to English." audio_s=1.740 wall_s=0.225 rtf=0.129
2025-08-16T18:10:09.480114Z  INFO runner: Voice command executed: Language set to to

And then set the language to "to"?