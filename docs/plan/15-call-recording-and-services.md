# 15 — Call Recording, CDR Pipeline & Media Services

## 1. P2P vs. Proxied Media & Selective Call Recording Architecture

### Media Routing Modes

RustPBX supports two distinct media processing modes controlled dynamically per-call, per-subscriber, or per-trunk:

```text
MODE A: Direct P2P Media (Default for local endpoints)
┌──────────────┐                                      ┌──────────────┐
│ SIP Phone A  │══════════ Direct RTP / SRTP ═════════►│ SIP Phone B  │
└──────┬───────┘                                      └──────┬───────┘
       │                                                     │
       └────────────── SIP Signaling via sip-engine ─────────┘

MODE B: Proxied / Recorded Media (Selective via media-node)
┌──────────────┐           ┌──────────────────┐           ┌──────────────┐
│ SIP Phone A  │══ RTP ═══►│    media-node    │══ RTP ═══►│ SIP Phone B  │
└──────┬───────┘           │  (RTP Relay &    │           └──────┬───────┘
       │                   │   Audio Recorder)│                  │
       │                   └────────┬─────────┘                  │
       │                            │                            │
       │                            ▼                            │
       └────── SIP Signaling ───────┴────── SIP Signaling ───────┘
                                    │
                                    ▼ Audio Encoders (WAV, Opus, MP3, GSM)
                           ┌──────────────────┐
                           │ Local Storage    │
                           │ /var/lib/...     │
                           └────────┬─────────┘
                                    │
                                    ▼ File path & metadata
                           ┌──────────────────┐
                           │ ClickHouse CDR   │
                           └──────────────────┘
```

---

## 2. Audio Recording Formats & Local Storage Layout

`media-node` supports multiple configurable audio formats per recording profile:

| Format | Audio Quality | Compression Ratio | CPU Overhead | Target Use Case |
|:---|:---|:---|:---|:---|
| **WAV (PCM)** | Lossless (Uncompressed) | 1x (Highest disk space) | Minimal (~0%) | High-fidelity archive, immediate processing |
| **Opus** | Excellent (Adaptive) | ~10x (Low disk space) | Moderate | WebRTC playback, modern standard |
| **MP3** | Very Good | ~8x | Moderate | Legacy browser & mobile playback |
| **GSM** | Telecom standard (8kHz) | ~13x | Low | Telephony archive, low storage footprint |

### Filesystem Structure

Recordings are written to local storage in a date-structured directory tree:

```text
/var/lib/rustpbx/recordings/
├── 2026/
│   └── 08/
│       └── 05/
│           ├── call-550e8400-legA-100.opus
│           ├── call-550e8400-legB-200.opus
│           └── call-550e8400-mixed.opus
```

---

## 3. Dynamic Recording Trigger Rules

Call recording is **disabled by default (P2P mode)** to maximize performance. It is dynamically proxied through `media-node` when triggered by:

1. **Subscriber Config**: `record_calls = true` in subscriber settings (SQLite).
2. **Trunk Config**: `record_calls = true` on outbound/inbound PSTN trunks (SQLite).
3. **REST API Command**: `POST /api/v1/calls/{id}/record/start` invoked on-demand via REST/gRPC.
4. **WASM Tier 1 Rule**: Custom WASM plugin evaluates headers (e.g., specific destination number or VIP flag) and enables recording in `<1ms`.

---

## 4. Modular Service Breakdown

To preserve memory safety, zero-copy performance, and fault isolation, each concern is implemented as a dedicated module or binary:

| Service / Concern | Module / Binary Location | Responsibilities |
|:---|:---|:---|
| **P2P / Direct SDP** | `sip-engine/b2bua/sdp_rewriter.rs` | Leaves media IP addresses untouched between endpoints when no recording/transcoding is required. |
| **Call Recording Engine** | `media-node/recorder/` | Taps active audio streams, decodes RTP payloads, encodes into chosen format (WAV, Opus, MP3, GSM), writes to local filesystem. |
| **CDR Pipeline** | `sip-engine/cdr/` & `ClickHouse` | Ring buffer collects call state events (`CallStarted`, `CallAnswered`, `CallEnded`), calculates duration/billing, batches into ClickHouse NoSQL database every 5s. |
| **WebRTC Gateway** | `bins/webrtc-gateway/` (Independent Binary) | Terminates WSS, ICE/STUN/TURN, DTLS-SRTP, translates WebRTC SDP to standard SIP SDP. |
| **SIP Packet Trace** | `sipstack/transport/inspector.rs` | Hooks into incoming/outgoing SIP messages, logs raw SIP text or exports to HEP/HOMER protocol for diagnostic capture. |
