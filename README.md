<div align="center">

# 🚀 RustPBX

**Next-Generation, Distributed, Extensible SIP PBX Built 100% in Pure Rust**

[![Rust](https://img.shields.io/badge/Language-Pure%20Rust%202021-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen.svg?style=for-the-badge&logo=github-actions)](https://github.com/mahirgul/rustpbx)
[![Architecture](https://img.shields.io/badge/Architecture-Process--Isolated%20Microservices-purple.svg?style=for-the-badge)](#-architecture-overview)
[![WASM](https://img.shields.io/badge/Plugins-WASM%20%2B%20gRPC-success.svg?style=for-the-badge&logo=webassembly)](docs/plan/08-plugin-system.md)

<p align="center">
  <a href="#-key-features">Key Features</a> •
  <a href="#-architecture-overview">Architecture</a> •
  <a href="#-implementation-status">Current Status</a> •
  <a href="#-documentation-map">Documentation</a> •
  <a href="#-getting-started">Getting Started</a> •
  <a href="#-benchmark-comparison">Comparison</a>
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
- 🔐 **RFC 2617 Digest Authentication**: Built-in MD5 challenge/response security flow (401 Unauthorized -> 200 OK) with per-extension toggles.
- ⚙️ **Per-Extension Fine-Tuning Settings**:
  - SIP OPTIONS qualify ping frequency (`qualify_frequency`)
  - NAT traversal modes (`auto`, `force_rport`, `stun`, `disabled`)
  - Min/Max Expiration time enforcement (`min_expires` / `max_expires`)
  - Max concurrent device logins & allowed transport protocol filters (`udp,tcp,tls,ws`)
- 🌐 **Embedded Standalone Web Admin UI**: Offline single-page dashboard embedded into binary via `rust-embed` with **Zero CDN/Internet dependencies**.
- 🔍 **Real-Time Health Monitoring**: Real OS tasklist process checks and TCP/UDP socket binding detection.
- 📊 **SQLite WAL Mode & Auto-Migrations**: Self-healing SQLite schema auto-migrations on boot (`sqlite:data/rustpbx.db`) with UNIX epoch saniyesi indexing.
- 📁 **Dual Console & File Logging**: Non-blocking `tracing-appender` writing structured logs to `logs/rustpbx.log`.

---

## 🏗️ Architecture Overview

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
```

---

## 🚦 Implementation Status

| Component | Status | Details |
|:---|:---:|:---|
| **`sipcore`** | ✅ Complete | Zero-I/O SIP parser (`nom` + `bytes`), RFC 2617 Digest Auth generator. |
| **`sdpcore`** | ✅ Complete | Pure SDP parser & RFC 3264 offer/answer negotiator. |
| **`sipstack`** | ✅ Complete | Tokio UDP/TCP/TLS/WebSocket transport, transaction timers, dialog ID tracks. |
| **`pbx-proto`** | ✅ Complete | Protobuf gRPC contracts (`media_control.proto`, `call_events.proto`, etc.). |
| **`sip-engine`** | ✅ Complete | B2BUA signaling engine, Digest Auth, REGISTER/OPTIONS/INVITE/BYE/CANCEL handling, SQLite WAL store. |
| **`media-node`** | ✅ Complete | gRPC MediaControl service, symmetric RTP relay loop, audio recording. |
| **`web-admin`** | ✅ Complete | Embedded offline UI (`rust-embed`), REST API, Extension CRUD, live registration & OS process health monitoring. |

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

# Run Core SIP Engine (Default Port 5060 UDP, API 8085)
cargo run --bin sip-engine

# Run Standalone Offline Web Admin UI (Port 8088 HTTP)
cargo run --bin web-admin
```

Open your browser at `http://localhost:8088` to access the Web Admin Dashboard.

---

## 📜 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.
