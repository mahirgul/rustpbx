# ARCHITECTURE_BRAIN.md — Project Brain & Architectural Knowledge Base

This document serves as the central design memory, architectural decision record (ADR), and technical knowledge base for **RustPBX**. All contributors and AI agents must abide by the architectural decisions documented here.

---

## 1. Core Architectural Mandates

1. **100% Pure Rust**: Zero C/C++ runtime or build dependencies. Networking via `tokio`, TLS via `rustls`, parsing via `nom` and `bytes`. Static binary compilation.
2. **100% English Standard**: All code, documentation, inline comments, variable names, error messages, and log output **MUST** be in English.
3. **Strict File Size & Modularity Limits**:
   - Source files **MUST NOT** exceed **300-400 lines of code**.
   - Large modules must be decomposed into granular submodules (e.g., individual header parsers in separate files under `headers/`).
4. **Mandatory Quality Gates**: Every build, pull request, and commit must pass cleanly without warnings:
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - `cargo test --workspace`
5. **Process Isolation & Crash Resilience**:
   - System functionality is partitioned into independent OS binaries: `sip-engine`, `media-node`, `webrtc-gateway`, `web-admin`, `call-center`, `voicemail`, `ai-agent`.
   - Crashes in media processing, WebRTC gateways, or AI streaming **never** interrupt core SIP signaling on `sip-engine`.

---

## 2. Media Path & Routing Strategy: P2P-First Hybrid Model

```text
DEFAULT (P2P Direct Relay Mode):
SIP Phone A  ◄══════════════ Direct RTP Stream ══════════════►  SIP Phone B
     │                                                               │
     └─────────────────── SIP Signaling via sip-engine ──────────────┘

TRIGGERED (Proxied / Recorded Media via media-node):
SIP Phone A  ═══ RTP ═══►  media-node (RTP Relay & Audio Recorder)  ═══ RTP ═══►  SIP Phone B
```

### Media Mode Decision Matrix

By default, calls execute in **Direct P2P Mode** (zero CPU/RTP overhead on PBX). `sip-engine` dynamically proxies media through `media-node` only when triggered by:
- **Call Recording Active**: Configured per extension, trunk, or REST API on-demand command.
- **WebRTC Interoperability**: Bridging WSS/DTLS-SRTP browser endpoints to standard UDP/RTP SIP phones.
- **IVR & Voicemail Sessions**: Playing system audio prompts or recording audio messages.
- **NAT Traversal Assistance**: Resolving symmetric NAT issues when endpoints cannot exchange P2P RTP streams.
- **Codec Transcoding**: Bridging mismatched audio codecs (e.g., Opus to PCMU).

---

## 3. Configuration Philosophy: TOML vs. SQLite Hybrid Model

### Rejected: Legacy Asterisk Text Configs (`extensions.conf`)
Asterisk-style text files (`sip.conf`, `extensions.conf`) are **explicitly rejected**. They suffer from spagetti syntax, fragile parsing, lack of relational querying, difficult web GUI integration, and risky runtime reloads.

### Adopted: Multi-Tier Hybrid Storage Model

| Storage Engine | Scope & Usage | Rationale & Access Pattern |
|:---|:---|:---|
| **TOML Files** | System, Network & Boot Settings (`sip-engine.toml`, `media-node.toml`) | Static boot parameters, bind IP/ports, log filters. Easily edited via `nano`/`vim` or CI/CD pipelines. |
| **SQLite (WAL Mode)** | Dynamic Telephony Objects (Extensions, Trunks, Dialplan Rules, IVRs, Schedules) | Relational integrity. Read into `sip-engine` RAM index on startup; modified dynamically via `web-admin` with zero-latency in-memory cache updates. |
| **ClickHouse** | Call Detail Records (CDR) & Telephony Analytics | Columnar NoSQL data store. Ultra-high compression (>85%) and sub-second analytical queries across millions of CDR rows. |
| **Local Filesystem** | Audio Recordings (`/var/lib/rustpbx/recordings/YYYY/MM/DD/`) | Configurable recording formats: **WAV** (lossless), **Opus** (WebRTC standard), **MP3**, or **GSM** (telephony archive). File paths referenced in ClickHouse CDR records. |

---

## 4. Sub-System & Binary Division

| Component | Execution Model | Primary Responsibility |
|:---|:---|:---|
| `sipcore` | Crate (Pure Library) | Zero-I/O SIP parser & AST types (`wasm32-wasi` compatible). |
| `sdpcore` | Crate (Pure Library) | Zero-I/O SDP parser & RFC 3264 offer/answer negotiator (`wasm32-wasi` compatible). |
| `sipstack` | Crate (Async Library) | Tokio-based UDP/TCP/TLS/WS transports, RFC 3261 transaction state machines, dialog tracking. |
| `pbx-proto` | Crate (Shared Protobuf) | Tonic/Prost gRPC definitions for inter-process communication. |
| `sip-engine` | Binary Executable | Core B2BUA signaling engine, CallManager, In-Process SBC pipeline, Axum REST API. |
| `media-node` | Binary Executable | RTP relay, Audio recorder (WAV/Opus/MP3/GSM), Audio Mixer, gRPC MediaControl service. |
| `webrtc-gateway` | Binary Executable | Edge WSS, DTLS-SRTP, ICE/STUN/TURN termination & SIP translation. |
| `web-admin` | Binary Executable | Unified Single-Pane-of-Glass web management dashboard, REST API, embedded DHCP & TFTP auto-provisioning servers. |
| `call-center` | Binary Executable | ACD queue distribution engine (`ring-all`, `round-robin`, `least-recent`), agent state tracking, queue callbacks. |
| `voicemail` | Binary Executable | Missed call message recording, MWI (RFC 3842) NOTIFY triggers, async SMTP email delivery. |
| `ai-agent` | Binary Executable | Real-time AI voice agents, live dual-channel STT/TTS streaming, LLM integration, barge-in support. |

---

## 5. Session Border Controller (SBC) Pipeline

Implemented in-process within `sip-engine` (`bins/sip-engine/src/sbc/`) via `sipstack`'s zero-copy `MessageInspector` pipeline to eliminate IPC overhead:
- **Topology Hiding**: Strips internal private IPs (`10.x.x.x`, `192.168.x.x`) from `Via`, `Record-Route`, and `Contact` headers. Replaces them with the public SBC address.
- **NAT Traversal (RFC 3581 `rport`)**: Appends `received` and `rport` parameters to incoming `Via` headers matching actual source socket addresses.
- **Header Normalization**: Adds/removes headers per trunk rules and strips revealing `User-Agent`/`Server` tokens.
- **SDP Rewriting**: Modifies SDP media connection IPs and ports to point to `media-node`.

---

## 6. Complete Document Index (`docs/plan/`)

- [00-mission.md](00-mission.md) — Mission, vision & 5 pillars
- [01-workspace-structure.md](01-workspace-structure.md) — Cargo workspace tree & dependency graph
- [02-sipcore-design.md](02-sipcore-design.md) — `sipcore` parser & `Bytes` zero-copy memory model
- [03-sdpcore-design.md](03-sdpcore-design.md) — `sdpcore` parser & offer/answer negotiator
- [04-sipstack-design.md](04-sipstack-design.md) — Transport, transaction state machines & dialogs
- [05-distributed-architecture.md](05-distributed-architecture.md) — Distributed process topology & IPC protocols
- [06-binary-roles.md](06-binary-roles.md) | `sip-engine` B2BUA call bridge model & satellite binary roles
- [07-fault-recovery.md](07-fault-recovery.md) — Graceful shutdown flow, WAL snapshots & heartbeat monitoring
- [08-plugin-system.md](08-plugin-system.md) — Two-tiered plugin architecture (WASM + gRPC)
- [09-storage.md](09-storage.md) — Multi-tier storage (TOML, SQLite, ClickHouse, Local Audio)
- [10-rfc-matrix.md](10-rfc-matrix.md) — RFC coverage roadmap & tech stack summary
- [11-implementation-plan.md](11-implementation-plan.md) — 32-week phased implementation roadmap
- [12-deployment-model.md](12-deployment-model.md) — Single-binary deployment & Asterisk/FreeSWITCH comparison
- [13-rest-api.md](13-rest-api.md) — REST API call control, WebSocket events & security
- [14-coding-standards.md](14-coding-standards.md) — Coding standards: English-only, file size limits (<400 lines), clippy & fmt
- [15-call-recording-and-services.md](15-call-recording-and-services.md) — Selective P2P vs. proxied recording & CDR pipeline
- [16-ivr-and-voicemail.md](16-ivr-and-voicemail.md) — IVR menu engine, DTMF collection, standalone Voicemail binary, MWI & SMTP
- [17-ai-integration.md](17-ai-integration.md) — Real-time AI voice agents, live STT/TTS streaming, LLMs & barge-in support
- [18-tts-service.md](18-tts-service.md) — Dynamic Text-to-Speech synthesis service, caching layer & multi-provider traits
- [19-web-admin-and-integrations.md](19-web-admin-and-integrations.md) — Unified Web Admin service & DB/HTTP/Email integrations
- [20-sbc-module.md](20-sbc-module.md) — In-process SBC pipeline, topology hiding, NAT rport traversal & header normalization
- [21-call-center-queues.md](21-call-center-queues.md) — Call Center Queue service (ACD), agent states & callbacks
- [22-conference-bridge.md](22-conference-bridge.md) — Multi-party audio mixing engine, N-1 summer, PIN auth & active speaker detection
- [23-security-and-toll-fraud.md](23-security-and-toll-fraud.md) — Intrusion detection, brute-force IP ban engine & toll fraud protection rules
- [24-time-based-routing.md](24-time-based-routing.md) — Business hours, holiday override calendars & manual time condition routing
- [25-fax-over-ip.md](25-fax-over-ip.md) — T.38 FoIP gateway, Fax-to-Email PDF conversion & Email-to-Fax
- [26-observability-and-monitoring.md](26-observability-and-monitoring.md) — Prometheus metrics, OpenTelemetry distributed tracing & HOMER HEP capture
- [27-auto-provisioning.md](27-auto-provisioning.md) — IP Phone Auto-Provisioning for Yealink, Grandstream, Fanvil, Snom, Cisco & Polycom
- [28-dhcp-and-tftp-servers.md](28-dhcp-and-tftp-servers.md) — Embedded Pure Rust DHCP (Option 66/160) & TFTP servers for zero-touch deployment
