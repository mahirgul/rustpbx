# 17 — AI Integration & Real-Time Voice Agent Architecture

## 1. Executive Summary & Vision

The AI Integration module brings modern artificial intelligence capabilities directly to **RustPBX**, enabling:

- **Real-Time AI Voice Agents**: Conversational AI bots answering calls in real-time (sub-500ms audio latency).
- **Speech-to-Text (STT)**: Real-time live call transcription and automated voicemail transcriptions.
- **Text-to-Speech (TTS)**: Dynamic neural voice prompt generation.
- **Sentiment & Intent Analysis**: Post-call or live conversation analytics.

---

## 2. Process & IPC Architecture

Following our process isolation principle, AI operations (which can be heavy on GPU/network I/O or HTTP streaming) run in an independent binary: `bins/ai-agent/`.

```text
┌──────────────┐                 ┌──────────────────┐                 ┌──────────────┐
│ SIP Phone /  │══ SIP / RTP ═══►│   media-node     │══ Audio Stream ═►│   ai-agent   │
│ WebRTC Client│                 │  (Audio Proxy)   │  (PCM / Opus)   │   (.exe)     │
└──────────────┘                 └────────┬─────────┘                 └──────┬───────┘
                                          │                                  │
                                          │ gRPC Events                      │ HTTP / WS
                                          ▼                                  ▼
                                 ┌──────────────────┐                ┌────────────────┐
                                 │   sip-engine     │                │ AI Providers   │
                                 │  (Call Control)  │                │ (OpenAI, Deep  │
                                 └──────────────────┘                │  gram, Whisper,│
                                                                     │  ElevenLabs,   │
                                                                     │  Local Ollama) │
                                                                     └────────────────┘
```

---

## 3. Core Features & Components

### A. Real-Time Streaming Audio Gateway (`bins/ai-agent/src/stream.rs`)
- Establishes a low-latency audio pipe directly with `media-node` via gRPC / UDP socket.
- Converts 8kHz/16kHz PCM audio streams into WebSocket payloads for real-time AI audio APIs (e.g., OpenAI Realtime API, Deepgram Flux, WhisperLive).

### B. Live Call Transcription (STT) (`bins/ai-agent/src/stt.rs`)
- Streams dual-channel (Caller & Callee) audio to STT services.
- Emits real-time transcription events to `sip-engine` via gRPC, which are streamed to the REST/WebSocket API (`/api/v1/events`).

### C. Conversational AI Voice Bot (`bins/ai-agent/src/bot.rs`)
- Acts as a virtual call endpoint.
- Integrates with LLMs (OpenAI, Anthropic Claude, Gemini, Local Llama via Ollama) and TTS engines (ElevenLabs, Piper TTS).
- Manages barge-in (interrupt handling): If the caller speaks while the AI bot is generating speech, `ai-agent` immediately cancels audio playback.

---

## 4. REST API & Event Integrations

### Trigger AI Agent via REST API

```http
POST /api/v1/calls/{call_id}/ai-agent
Content-Type: application/json
Authorization: Bearer <api-key>

{
  "provider": "openai-realtime",
  "system_prompt": "You are a customer service assistant for RustPBX. Be concise and friendly.",
  "voice": "alloy",
  "transcribe_live": true
}
```

### Real-Time WebSocket Event Stream

```json
{
  "event": "ai.transcription",
  "call_id": "550e8400-...",
  "speaker": "caller",
  "text": "Hello, I would like to check my account balance.",
  "confidence": 0.98,
  "timestamp": "2026-08-05T16:01:00Z"
}
```

---

## 5. Implementation Phase Schedule

AI integration is scheduled in **Phase 4 (Weeks 29-31)**:

- **Week 29**: `ai-agent` binary foundation, audio stream pipe with `media-node`.
- **Week 30**: STT streaming (Whisper / Deepgram) & automated Voicemail transcription.
- **Week 31**: Conversational Voice AI Bot engine with barge-in support & REST API triggers.
