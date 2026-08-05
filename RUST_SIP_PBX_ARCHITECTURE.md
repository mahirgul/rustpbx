# RustPBX — PURE RUST DISTRIBUTED & EXTENSIBLE SIP PBX

A next-generation SIP PBX built with **100% Pure Rust**, designed to replace legacy, monolithic C/C++ PBX software like Asterisk and FreeSWITCH.

---

## 5 Core Pillars

| # | Pillar | Description |
|---|---|---|
| 🛡️ | **100% Pure Rust** | Zero C/C++ external dependencies; compiles to static single binaries |
| 🧱 | **Process Isolation** | Independent executables per binary (`sip-engine`, `media-node`, `webrtc-gateway`, `ai-agent`, `web-admin`, `call-center`, `voicemail`) |
| ⚡ | **Two-Tiered Plugins** | WASM Tier 1 (<1ms, sandboxed) + gRPC Tier 2 (any language) |
| 📦 | **Single-Binary Deployment** | Static link, zero runtime dependencies, simple `scp` deployment |
| 🏗️ | **Three-Crate Foundation** | Clear layer split: `sipcore` (parser) → `sdpcore` (SDP) → `sipstack` (async stack) |

---

## Project Plan Documents (`docs/plan/`)

The full architectural blueprint is split into modular files under `docs/plan/`:

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
| 09 | [Storage & Management](docs/plan/09-storage.md) | Storage architecture & management API |
| 10 | [RFC Matrix](docs/plan/10-rfc-matrix.md) | RFC coverage roadmap (RFC 3261, 3262, 3515, etc.) & tech stack |
| 11 | [Implementation Plan](docs/plan/11-implementation-plan.md) | 32-week phased implementation roadmap |
| 12 | [Deployment Model](docs/plan/12-deployment-model.md) | Deployment topologies & Asterisk/FreeSWITCH comparison |
| 13 | [REST API](docs/plan/13-rest-api.md) | Call control endpoints, WebSocket events & security |
| 14 | [Coding Standards](docs/plan/14-coding-standards.md) | English-only, modular file limits (<400 lines), clippy & fmt rules |
| 15 | [Call Recording & Services](docs/plan/15-call-recording-and-services.md) | Selective P2P vs. proxied audio recording, CDR pipeline, WebRTC & SIP tracing |
| 16 | [IVR & Voicemail](docs/plan/16-ivr-and-voicemail.md) | IVR menu engine, DTMF collection, standalone Voicemail binary, MWI & SMTP |
| 17 | [AI Integration](docs/plan/17-ai-integration.md) | Real-time AI voice agents, live STT/TTS streaming, LLMs & barge-in support |
| 18 | [TTS Service](docs/plan/18-tts-service.md) | Dynamic Text-to-Speech synthesis service, caching layer & multi-provider traits |
| 19 | [Unified Web Admin & Integrations](docs/plan/19-web-admin-and-integrations.md) | Unified Web Admin service (PBX, IVR, Voicemail) & DB/HTTP/Email integrations |
| 20 | [SBC Module Architecture](docs/plan/20-sbc-module.md) | In-process SBC pipeline, topology hiding, NAT rport traversal & header normalization |
| 21 | [Call Center Queues](docs/plan/21-call-center-queues.md) | Call Center Queue service (ACD), agent states, distribution strategies & callbacks |
| 22 | [Conference Bridge](docs/plan/22-conference-bridge.md) | Multi-party audio mixing engine, N-1 summer, PIN auth & active speaker detection |
| 23 | [Security & Toll Fraud](docs/plan/23-security-and-toll-fraud.md) | Intrusion detection, brute-force IP ban engine & toll fraud protection rules |
| 24 | [Time-Based Routing](docs/plan/24-time-based-routing.md) | Business hours, holiday override calendars & manual time condition routing |
| 25 | [Fax over IP (T.38)](docs/plan/25-fax-over-ip.md) | T.38 FoIP gateway, Fax-to-Email PDF conversion & Email-to-Fax |
| 26 | [Observability & Metrics](docs/plan/26-observability-and-monitoring.md) | Prometheus metrics, OpenTelemetry distributed tracing & HOMER HEP packet capture |
| 27 | [Auto-Provisioning System](docs/plan/27-auto-provisioning.md) | IP Phone Auto-Provisioning for Yealink, Grandstream, Fanvil, Snom, Cisco & Polycom |
| 28 | [DHCP & TFTP Servers](docs/plan/28-dhcp-and-tftp-servers.md) | Embedded Pure Rust DHCP (Option 66/160) & TFTP servers for zero-touch deployment |

---

## Technical Standards & Guidelines

- **Language**: 100% English for code, comments, documentation, and logs.
- **Modularity**: Files must be kept small (target <300-400 lines) and strictly divided into submodules.
- **Quality**: `cargo fmt` and `cargo clippy -- -D warnings` must pass cleanly on every build.
