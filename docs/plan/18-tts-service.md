# 18 — Standalone TTS (Text-to-Speech) Service Architecture

## 1. Executive Summary & Purpose

The **TTS Service** (`bins/tts-service/` or `media-node/tts/`) provides dynamic voice synthesis for **RustPBX**, allowing IVR menus, voicemail greetings, dynamic prompts (e.g., account balance, caller names, queue position announcements), and AI agents to convert text into natural speech audio streams.

---

## 2. Supported TTS Providers

The architecture is provider-agnostic, supporting both cloud APIs and offline local neural speech models via a unified Rust trait:

| Provider | Type | Latency | Target Use Case |
|:---|:---|:---|:---|
| **Piper TTS** | Offline / Local (Pure C++/WASM or Native) | Zero network (<50ms) | Low latency, zero cost, privacy-first |
| **ElevenLabs** | Cloud API | ~200-400ms | Ultra-realistic neural voices |
| **Google Cloud Text-to-Speech** | Cloud API | ~150-300ms | Multi-lingual support (>220 voices) |
| **Amazon Polly** | Cloud API | ~150-250ms | Standard telephony synthesis (NTTS) |
| **Azure Speech Services** | Cloud API | ~150-250ms | Custom neural voice support |

---

## 3. Architecture & Audio Caching Pipeline

```text
sip-engine / IVR ──gRPC: Synthesize(text, voice, format)──► tts-service
                                                               │
                                                               ├── 1. Compute SHA-256 hash of (text + voice + format)
                                                               ├── 2. Check local disk cache (/var/lib/rustpbx/tts_cache/)
                                                               │      └─► If CACHE HIT: Return cached .opus/.wav audio immediately (<5ms)
                                                               ├── 3. If CACHE MISS: Dispatch to configured TTS Provider API
                                                               ├── 4. Convert output to target telephony PCM/Opus stream
                                                               ├── 5. Save to disk cache asynchronously
                                                               └── 6. Return audio stream to media-node / sip-engine
```

---

## 4. Unified Rust Trait Definition

```rust
use async_trait::async_trait;
use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct TtsRequest {
    pub text: String,
    pub voice_id: String,
    pub language_code: String,
    pub sample_rate_hz: u32,
    pub format: AudioFormat, // PCM16, Opus, WAV
}

#[async_trait]
pub trait TtsProvider: Send + Sync {
    /// Synthesize text to audio bytes
    async fn synthesize(&self, req: &TtsRequest) -> Result<Bytes, TtsError>;
    
    /// Unique identifier of provider (e.g., "piper", "elevenlabs")
    fn name(&self) -> &'static str;
}
```

---

## 5. Integration Points

- **IVR Prompts**: Dynamic prompts like *"Your position in queue is 3. Estimated wait time is 2 minutes"* generated on the fly.
- **Voicemail Greetings**: Automatic personal greetings *"You have reached 555-0199. Please leave a message after the tone"*.
- **AI Voice Agent Integration**: Works alongside `ai-agent` for streaming text response output directly into callers' ears.

---

## 6. Implementation Phase

Scheduled for **Phase 3 (Weeks 21-22)** alongside IVR and Event Framework enhancements.
