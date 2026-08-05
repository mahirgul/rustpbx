# RustPBX Plan — Executive Summary & Index

This directory contains the complete architectural specification, design documents, and implementation roadmap for **RustPBX** — a high-performance, fault-tolerant, pure Rust distributed SIP PBX.

---

## Document Map

| # | Document | Description |
|---|---|---|
| 00 | [00-mission.md](00-mission.md) | Mission, vision, and 5 core pillars |
| 01 | [01-workspace-structure.md](01-workspace-structure.md) | Cargo workspace structure & dependency graph |
| 02 | [02-sipcore-design.md](02-sipcore-design.md) | `sipcore` design — pure parser, memory model & 3-stage parsing |
| 03 | [03-sdpcore-design.md](03-sdpcore-design.md) | `sdpcore` design — SDP parsing & negotiation |
| 04 | [04-sipstack-design.md](04-sipstack-design.md) | `sipstack` design — transport, transaction & dialog layers |
| 05 | [05-distributed-architecture.md](05-distributed-architecture.md) | Distributed architecture, IPC & media path coordination |
| 06 | [06-binary-roles.md](06-binary-roles.md) | Binary roles & B2BUA call bridge model |
| 07 | [07-fault-recovery.md](07-fault-recovery.md) | Graceful shutdown & crash recovery snapshots |
| 08 | [08-plugin-system.md](08-plugin-system.md) | Two-tiered plugin architecture (WASM + gRPC) |
| 09 | [09-storage.md](09-storage.md) | Storage architecture & management API |
| 10 | [10-rfc-matrix.md](10-rfc-matrix.md) | RFC coverage matrix & tech stack summary |
| 11 | [11-implementation-plan.md](11-implementation-plan.md) | 32-week phased implementation roadmap |
| 12 | [12-deployment-model.md](12-deployment-model.md) | Deployment model & Asterisk/FreeSWITCH comparison |
| 13 | [13-rest-api.md](13-rest-api.md) | REST API call control, WebSocket events & security |
| 14 | [14-coding-standards.md](14-coding-standards.md) | Coding standards: English-only, modular file limits (<400 lines), clippy & fmt |
| 15 | [15-call-recording-and-services.md](15-call-recording-and-services.md) | Selective P2P vs. proxied audio recording, CDR pipeline, WebRTC & SIP tracing |
| 16 | [16-ivr-and-voicemail.md](16-ivr-and-voicemail.md) | IVR menu engine, DTMF collection, standalone Voicemail binary, MWI & SMTP |
| 17 | [17-ai-integration.md](17-ai-integration.md) | Real-time AI voice agents, live STT/TTS streaming, LLMs & barge-in support |
| 18 | [18-tts-service.md](18-tts-service.md) | Dynamic Text-to-Speech synthesis service, caching layer & multi-provider traits |
| 19 | [19-web-admin-and-integrations.md](19-web-admin-and-integrations.md) | Unified Web Admin service (PBX, IVR, Voicemail) & DB/HTTP/Email integrations |
| 20 | [20-sbc-module.md](20-sbc-module.md) | In-process SBC pipeline, topology hiding, NAT rport traversal & header normalization |
| 21 | [21-call-center-queues.md](21-call-center-queues.md) | Call Center Queue service (ACD), agent states, distribution strategies & callbacks |
| 22 | [22-conference-bridge.md](22-conference-bridge.md) | Multi-party audio mixing engine, N-1 summer, PIN auth & active speaker detection |
| 23 | [23-security-and-toll-fraud.md](23-security-and-toll-fraud.md) | Intrusion detection, brute-force IP ban engine & toll fraud protection rules |
| 24 | [24-time-based-routing.md](24-time-based-routing.md) | Business hours, holiday override calendars & manual time condition routing |
| 25 | [25-fax-over-ip.md](25-fax-over-ip.md) | T.38 FoIP gateway, Fax-to-Email PDF conversion & Email-to-Fax |
| 26 | [26-observability-and-monitoring.md](26-observability-and-monitoring.md) | Prometheus metrics, OpenTelemetry distributed tracing & HOMER HEP packet capture |
| 27 | [27-auto-provisioning.md](27-auto-provisioning.md) | IP Phone Auto-Provisioning for Yealink, Grandstream, Fanvil, Snom, Cisco & Polycom |
| 28 | [28-dhcp-and-tftp-servers.md](28-dhcp-and-tftp-servers.md) | Embedded Pure Rust DHCP (Option 66/160) & TFTP servers for zero-touch deployment |

---

## Key Core Guarantees

1. **Pure Rust**: Zero C/C++ dependencies (`rustls`, `tokio`, `nom`, `bytes`).
2. **Process Isolation**: Each service runs as an independent executable binary.
3. **Modular Codebase**: Files are kept small (<300-400 lines) and strictly divided into submodules.
4. **Strict Quality**: `cargo fmt` and `cargo clippy -- -D warnings` must pass on every build.
5. **English Only**: All code, documentation, comments, and logs are strictly in English.
