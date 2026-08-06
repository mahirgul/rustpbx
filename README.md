<div align="center">

# 🚀 RustPBX

**Next-Generation, Distributed, Extensible SIP B2BUA & WebRTC Telecommunications Platform Built 100% in Pure Rust**

[![Rust](https://img.shields.io/badge/Language-Pure%20Rust%202021-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen.svg?style=for-the-badge&logo=github-actions)](https://github.com/mahirgul/rustpbx)
[![Architecture](https://img.shields.io/badge/Architecture-Decoupled%206--Layer-purple.svg?style=for-the-badge)](#-decoupled-6-layer-architecture)
[![Logging](https://img.shields.io/badge/Logging-Modular%20Dedicated%20Files-success.svg?style=for-the-badge)](#-modular-logging-system)

<p align="center">
  <a href="#-key-features">Key Features</a> •
  <a href="#-decoupled-6-layer-architecture">Architecture</a> •
  <a href="#-implementation-status">Current Status</a> •
  <a href="#-modular-logging-system">Modular Logging</a> •
  <a href="#-getting-started">Getting Started</a> •
  <a href="#-documentation-map">Documentation</a>
</p>

---

</div>

## 📌 Vision & Mission

**RustPBX** is designed to replace legacy, monolithic C/C++ PBX software (such as Asterisk and FreeSWITCH) with a modern, memory-safe, event-driven, fault-tolerant telecommunications engine written entirely in **100% Pure Rust**.

By separating network signaling, media processing, WebRTC edge termination, and administration into **independent OS executables**, a failure or crash in one satellite process never disrupts active calls or core SIP signaling.

---

## ✨ Key Features

- 🛡️ **100% Pure Rust**: Zero external C/C++ dependencies (`rustls`, `tokio`, `nom`, `bytes`). Compiles into static, self-contained single binaries.
- 🧱 **Process Isolation**: Fault-tolerant micro-binary design (`sip-engine`, `media-node`, `web-admin`, `webrtc-gateway`).
- ⚡ **Decoupled 6-Layer Architecture**: Clean separation between Transport, Transaction, Dialog, Call Engine, Media Engine, and Application Services.
- 🔐 **RFC 2617 / 3261 Digest Authentication**: Method-dynamic MD5 challenge/response security (`REGISTER`, `INVITE`) with per-extension toggles.
- 📜 **Strict RFC 3261 Compliance**:
  - `ACK` handling without illegal response generation (§17.2.2)
  - `To` tag dialog parameter generation on non-100 responses (§8.2.6.2)
  - Requested `Expires` clamping & `Expires: 0` instant unregistration/logout handling (§10.2.2)
  - Active callee registration lookups with `480 Temporarily Unavailable` fallbacks for offline endpoints
- 🔀 **Full Dialplan Rule Management**: Regex pattern matching, target routing (`extension`, `trunk`, `ivr`, `queue`, `voicemail`), and full CRUD (Create, List, Edit via PUT API/Modal, Delete) with automatic database deduplication.
- 🌐 **WebRTC Edge Border Gateway (`webrtc-gateway`)**: Dedicated WebSocket WSS (`8089`) signaling bridge translating WebRTC SDP to native SIP.
- 🔊 **Multi-Node Media Engine (`media-node`)**: Configurable wrap-around RTP port range allocations (`port_range_start..port_range_end`), symmetric NAT auto-learning, and WAV recording pool.
- 📂 **Modular Dedicated Logging**: Dedicated log files (`logs/sip_messages.log`, `logs/auth_audit.log`, `logs/system.log`, `logs/rustpbx.log`).
- 🌐 **Embedded Standalone Web Admin UI**: Offline single-page dashboard embedded into binary via `rust-embed` with SPA fallback routing and **Zero CDN/Internet dependencies**.
- 📊 **SQLite WAL Mode & Auto-Migrations**: Self-healing SQLite schema auto-migrations on boot (`sqlite:data/rustpbx.db`) with `UNIQUE(rule_name, pattern)` constraints.

---

## 🏗️ Decoupled 6-Layer Architecture

```text
                                     PUBLIC INTERNET
                                            │
                     ┌──────────────────────┴──────────────────────┐
                     │                                             │
          WSS / DTLS-SRTP (WebRTC)                          SIP (UDP/TCP/TLS)
                     │                                             │
                     ▼                                             ▼
       ┌──────────────────────────┐                  ┌──────────────────────────┐
       │   webrtc-gateway         │                  │   sip-engine             │
       │   (Edge Gateway)         │====[gRPC IPC]===>│   (Core B2BUA Engine)    │
       └─────────────┬────────────┘                  └─────────────┬────────────┘
                     │                                             │
                     │                                             │ (In-Process <1ms)
                     │                              [gRPC]         v
                     │                                 │   ┌───────────────────┐
                     │                                 v   │ WASM Plugins      │
                     │                  ┌──────────────┴───┤ (Tier 1 Fast-Path)│
                     │                  │  media-node      └───────────────────┘
                     └====[RTP/SRTP]===>│  (RTP Relay &    │
                                        │   Recording)     │
                                        └──────────────────┘
                                                 │
                                        [gRPC Async Events]
                                                 │
                                                 ▼
                                        ┌──────────────────┐
                                        │ web-admin        │
                                        │ (Management UI & │
                                        │  REST API)       │
                                        └──────────────────┘

    Layer 1: Transport Layer (UDP, TCP, TLS, WebSocket)
        ↓
    Layer 2: SIP Transaction Layer (ICT, IST, NICT, NIST)
        ↓
    Layer 3: Dialog Layer (Dialog State & Route Set)
        ↓
    Layer 4: Call Engine (B2BUA Call Bridge & Dialplan)
        ↓
    Layer 5: Media Engine (RTP Relay, SRTP & WAV Recorder)
        ↓
    Layer 6: Application Services (IVR, ACD Queues, Voicemail)
```

---

## 🚦 Implementation Status

| Component | Status | Details |
|:---|:---:|:---|
| **`sipcore`** | ✅ Complete | Zero-I/O SIP parser (`nom` + `bytes`), RFC 2617 Digest Auth generator. |
| **`sdpcore`** | ✅ Complete | Pure SDP parser & RFC 3264 offer/answer negotiator. |
| **`sipstack`** | ✅ Complete | Tokio UDP/TCP/TLS/WebSocket transport, transaction timers, dialog ID tracks. |
| **`pbx-proto`** | ✅ Complete | Protobuf gRPC contracts (`media_control.proto`, `call_events.proto`, etc.). |
| **`sip-engine`** | ✅ Complete | B2BUA signaling engine, dynamic Digest Auth, active callee lookup, REGISTER/OPTIONS/INVITE/BYE/CANCEL handling, SQLite WAL store. |
| **`media-node`** | ✅ Complete | gRPC MediaControl service, wrap-around port allocation (`port_range_start..end`), symmetric RTP relay loop, WAV audio recording. |
| **`webrtc-gateway`** | ✅ Complete | Dedicated WebSocket WSS (`8089`) signaling server bridging browser WebRTC SDP to core SIP engine. |
| **`web-admin`** | ✅ Complete | Embedded offline UI (`rust-embed`), REST API for Extensions/Trunks/Dialplan (Full Edit support), SPA fallback, live registration & ultra-fast TCP/UDP socket health monitoring. |

---

## 📂 Modular Logging System

RustPBX separates system events into dedicated log files under the `logs/` directory for fast troubleshooting:

- **`logs/sip_messages.log`**: Complete raw SIP packet wire tracing (`<<< INCOMING SIP`, `>>> OUTGOING SIP`) including full headers and SDP body.
- **`logs/auth_audit.log`**: Dedicated Digest Security audit trail (Successful logins, `401 Challenges`, failed attempts with IP & Port).
- **`logs/system.log`**: Process lifecycle events, DB migrations, REST API operations, and port bindings.
- **`logs/rustpbx.log`**: Master combined log stream.

---

## 🛠️ Getting Started

### Prerequisites

- [Rust Toolchain](https://www.rust-lang.org/tools/install) (1.75+ recommended)

### Building & Running

```bash
# Clone the repository
git clone https://github.com/mahirgul/rustpbx.git
cd rustpbx

# Verify Quality Gates (0 Warnings Mandate)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace

# Run Core SIP Engine (Default Port 5060 UDP, REST API 8085)
cargo run --bin sip-engine

# Run Media Node (Default gRPC 50051, RTP Ports 10000-20000)
cargo run --bin media-node

# Run WebRTC Gateway (Default WSS 8089)
cargo run --bin webrtc-gateway

# Run Standalone Offline Web Admin UI (Port 8088 HTTP)
cargo run --bin web-admin
```

Open your browser at `http://localhost:8088` to access the Web Admin Dashboard.

---

## 📜 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.
