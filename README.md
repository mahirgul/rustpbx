<div align="center">

# 🚀 RustPBX

**Next-Generation, Distributed, Extensible SIP PBX Built 100% in Pure Rust**

[![Rust](https://img.shields.io/badge/Language-Pure%20Rust%202021-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen.svg?style=for-the-badge&logo=github-actions)](https://github.com/)
[![Architecture](https://img.shields.io/badge/Architecture-Process--Isolated%20Microservices-purple.svg?style=for-the-badge)](#-architecture-overview)
[![WASM](https://img.shields.io/badge/Plugins-WASM%20%2B%20gRPC-success.svg?style=for-the-badge&logo=webassembly)](docs/plan/08-plugin-system.md)

<p align="center">
  <a href="#-key-features">Key Features</a> •
  <a href="#-architecture-overview">Architecture</a> •
  <a href="#-documentation-map">Documentation</a> •
  <a href="#-getting-started">Getting Started</a> •
  <a href="#-benchmark-comparison">Comparison</a> •
  <a href="#-contributing">Contributing</a>
</p>

---

</div>

## 📌 Vision & Mission

**RustPBX** is designed to replace legacy, monolithic C/C++ PBX software (such as Asterisk and FreeSWITCH) with a modern, memory-safe, event-driven, fault-tolerant telecommunications engine written entirely in **100% Pure Rust**.

By separating network signaling, media processing, WebRTC edge termination, and administration into **independent OS executables**, a failure or crash in one satellite process never disrupts active calls or core SIP signaling.

---

## ✨ Key Features

- 🛡️ **100% Pure Rust**: Zero external C/C++ dependencies (`rustls`, `tokio`, `nom`, `bytes`). Compiles into static, self-contained single binaries.
- 🧱 **Process Isolation**: Fault-tolerant micro-binary design (`sip-engine`, `media-node`, `webrtc-gateway`, `ai-agent`, `web-admin`, `call-center`, `voicemail`).
- ⚡ **Two-Tiered Extensibility**:
  - **Tier 1 (WASM)**: In-process sandboxed execution (<1ms latency) for high-speed routing & header filtering.
  - **Tier 2 (gRPC)**: Out-of-process event & control plugins writable in **any programming language** (Go, Python, C#, Node.js).
- 🌐 **WebRTC & Native SIP**: Seamless bridging between WebRTC browser clients (WSS, DTLS-SRTP, ICE) and traditional SIP endpoints/trunks.
- 🤖 **Real-Time Voice AI Integration**: Native support for conversational AI voice agents (OpenAI Realtime, Whisper STT, Piper TTS) with live barge-in support.
- 📊 **Enterprise Storage & Analytics**: SQLite (WAL Mode) for relational config, **ClickHouse** for high-compression CDR analytics, and local/S3 audio recording storage.
- 📱 **Zero-Touch Auto-Provisioning**: Embedded DHCP (Option 66/160) and TFTP servers supporting Yealink, Grandstream, Fanvil, Snom, Cisco, and Polycom IP phones.
- 📞 **Complete Telephony Feature Suite**:
  - ACD Call Center Queues with Callbacks
  - Multi-Party Audio Conference Bridge (N-1 Summer)
  - Interactive Voice Response (IVR) Engine with DB/HTTP Webhook Integration
  - Message Waiting Indicator (MWI) & Voicemail-to-Email (SMTP)
  - In-Process Session Border Controller (SBC) with Topology Hiding & NAT `rport` Traversal
  - T.38 Fax over IP (FoIP) Gateway
  - Intrusion Detection & Toll Fraud Protection

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
         ┌──────────────────────┬────────────────┴──────┬──────────────────────┐
         ▼                      ▼                       ▼                      ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ web-admin       │    │ call-center     │    │ voicemail       │    │ ai-agent        │
│ (Management)    │    │ (ACD Queues)    │    │ (MWI & SMTP)    │    │ (STT / LLM)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
```

---

## 📚 Documentation Map

The full technical specification is modularized into 28 detailed architectural documents under [`docs/plan/`](docs/plan/):

| # | Document | Content Summary |
|---|---|---|
| 00 | [Mission & Vision](docs/plan/00-mission.md) | Core goals, principles, and architectural vision |
| 01 | [Workspace Structure](docs/plan/01-workspace-structure.md) | Cargo workspace setup, directory layout, dependency graph |
| 02 | [sipcore Design](docs/plan/02-sipcore-design.md) | Pure SIP parser, `Bytes` zero-copy memory model, 3-stage parser |
| 03 | [sdpcore Design](docs/plan/03-sdpcore-design.md) | Pure SDP parser, offer/answer negotiation, glue logic |
| 04 | [sipstack Design](docs/plan/04-sipstack-design.md) | Transport, transaction state machines (RFC 3261), dialog layer |
| 05 | [Distributed Architecture](docs/plan/05-distributed-architecture.md) | Process topology, IPC protocols, WebRTC media path sequence |
| 06 | [Binary Roles](docs/plan/06-binary-roles.md) | `sip-engine` B2BUA call bridge model & satellite binary roles |
| 07 | [Crash Recovery](docs/plan/07-fault-recovery.md) | Graceful shutdown flow, WAL snapshots, heartbeat monitoring |
| 08 | [Plugin Architecture](docs/plan/08-plugin-system.md) | WASM sandbox (Tier 1) & gRPC microservices (Tier 2) |
| 09 | [Storage & Management](docs/plan/09-storage.md) | SQLite, ClickHouse, local audio storage & management port |
| 10 | [RFC Matrix](docs/plan/10-rfc-matrix.md) | RFC coverage roadmap (RFC 3261, 3262, 3515, etc.) & tech stack |
| 11 | [Implementation Plan](docs/plan/11-implementation-plan.md) | 32-week phased development roadmap with quality gates |
| 12 | [Deployment Model](docs/plan/12-deployment-model.md) | Single-binary deployment & Asterisk/FreeSWITCH comparison |
| 13 | [REST API](docs/plan/13-rest-api.md) | Call control endpoints, WebSocket events & security |
| 14 | [Coding Standards](docs/plan/14-coding-standards.md) | English-only, modular file limits (<400 lines), clippy & fmt rules |
| 15 | [Call Recording & Services](docs/plan/15-call-recording-and-services.md) | Selective P2P vs. proxied audio recording, CDR pipeline, WebRTC & SIP tracing |
| 16 | [IVR & Voicemail](docs/plan/16-ivr-and-voicemail.md) | IVR menu engine, DTMF collection, standalone Voicemail binary, MWI & SMTP |
| 17 | [AI Integration](docs/plan/17-ai-integration.md) | Real-time AI voice agents, live STT/TTS streaming, LLMs & barge-in support |
| 18 | [TTS Service](docs/plan/18-tts-service.md) | Dynamic Text-to-Speech synthesis service, caching layer & multi-provider traits |
| 19 | [Unified Web Admin](docs/plan/19-web-admin-and-integrations.md) | Unified Web Admin service (PBX, IVR, Voicemail) & DB/HTTP/Email integrations |
| 20 | [SBC Module Architecture](docs/plan/20-sbc-module.md) | In-process SBC pipeline, topology hiding, NAT rport traversal & header normalization |
| 21 | [Call Center Queues](docs/plan/21-call-center-queues.md) | Call Center Queue service (ACD), agent states, distribution strategies & callbacks |
| 22 | [Conference Bridge](docs/plan/22-conference-bridge.md) | Multi-party audio mixing engine, N-1 summer, PIN auth & active speaker detection |
| 23 | [Security & Toll Fraud](docs/plan/23-security-and-toll-fraud.md) | Intrusion detection, brute-force IP ban engine & toll fraud protection rules |
| 24 | [Time-Based Routing](docs/plan/24-time-based-routing.md) | Business hours, holiday override calendars & manual time condition routing |
| 25 | [Fax over IP (T.38)](docs/plan/25-fax-over-ip.md) | T.38 FoIP gateway, Fax-to-Email PDF conversion & Email-to-Fax |
| 26 | [Observability & Metrics](docs/plan/26-observability-and-monitoring.md) | Prometheus metrics (`/metrics`), OpenTelemetry distributed tracing & HOMER HEP capture |
| 27 | [Auto-Provisioning System](docs/plan/27-auto-provisioning.md) | IP Phone Auto-Provisioning for Yealink, Grandstream, Fanvil, Snom, Cisco & Polycom |
| 28 | [DHCP & TFTP Servers](docs/plan/28-dhcp-and-tftp-servers.md) | Embedded Pure Rust DHCP (Option 66/160) & TFTP servers for zero-touch deployment |

---

## 📊 Benchmark & Comparison

| Feature | Asterisk | FreeSWITCH | **RustPBX** |
|:---|:---|:---|:---|
| **Language** | C | C | **Pure Rust (2021)** |
| **Memory Safety** | ❌ Manual `malloc`/`free` | ❌ Manual memory management | ✅ **Guaranteed by Rust Ownership** |
| **Crash Impact** | 💀 Whole PBX crashes | 💀 Whole PBX crashes | 🛡️ **Isolated to single process** |
| **External Dependencies** | ~40 C libraries | ~30 C libraries | **0 (Pure Cargo Workspace)** |
| **Deployment** | `make install` + OS packages | `make install` + OS packages | **Single static binary via `scp`** |
| **Plugin Extensibility** | Dialplan + C modules (unsafe) | Lua/JS (unsafe in-process) | **WASM (sandboxed) + gRPC (any language)** |
| **WebRTC Architecture** | In-process module | Native (mod_verto) | **Isolated Border Gateway executable** |
| **Configuration** | Complex `.conf` files | Verbose XML | **Type-safe TOML & SQLite** |
| **Target Concurrent Calls**| ~1,000 | ~3,000 | **5,000+ (Tokio Async, Lock-Free)** |

---

## 🛠️ Getting Started

### Prerequisites

- [Rust Toolchain](https://www.rust-lang.org/tools/install) (1.75+ recommended)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/your-org/rustpbx.git
cd rustpbx

# Run format and linter quality checks
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build all release binaries
cargo build --release
```

### Binaries Generated

After a successful release build, executables are available in `target/release/`:

- `sip-engine`: Core B2BUA SIP signaling engine
- `media-node`: RTP relay, audio recording & mixing server
- `webrtc-gateway`: WebRTC edge border gateway
- `web-admin`: Management web service, REST API, DHCP & TFTP auto-provisioning
- `call-center`: ACD queue service
- `voicemail`: Missed call recording, MWI & email delivery service
- `ai-agent`: Real-time AI voice agent & STT streaming gateway

---

## 📜 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.
