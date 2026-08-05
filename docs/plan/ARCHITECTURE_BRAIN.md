# ARCHITECTURE_BRAIN.md — Project Brain & Architectural Knowledge Base

This file serves as the central context and design memory ("brain") for the **RustPBX** architecture. It records core decisions, structural constraints, memory models, and project rules.

---

## 1. Core Mandates

1. **100% Pure Rust**: Zero C/C++ libraries. Networking via `tokio`, TLS via `rustls`, parsing via `nom` and `bytes`.
2. **100% English**: All documentation, code, comments, variable names, and log messages must be in English.
3. **Strict Modularity & File Size Limit**:
   - Source files **MUST NOT** exceed 300-400 lines of code.
   - Large concepts must be broken into granular submodules (e.g., individual header parsers in separate files).
4. **Mandatory Linting & Formatting**:
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - Every build/commit must pass these checks without warnings.
5. **Fault Isolation**:
   - Core binaries run as separate OS processes: `sip-engine`, `media-node`, `webrtc-gateway`, `web-admin`, `call-center`, `voicemail`.
   - Crashes in satellite processes do not interrupt signaling on `sip-engine`.

---

## 2. SIP Architecture & Memory Model

- **Layer Separation**:
  - `crates/sipcore`: Pure SIP message & header parser + AST types. Zero I/O, no `tokio`. Compiles to `wasm32-wasi`.
  - `crates/sdpcore`: Pure SDP parser & offer/answer negotiator. Zero I/O, no dependency on `sipcore`. Compiles to `wasm32-wasi`.
  - `crates/sipstack`: Async transport (UDP/TCP/TLS/WS), RFC 3261 transaction state machines (ICT, NICT, IST, NIST), and dialog tracking. Depends on `sipcore` and `sdpcore`.
- **Zero-Copy Slicing**:
  - `SipMessage` holds a `bytes::Bytes` handle.
  - Headers store byte range offsets into the main buffer.
  - Typed header parsing is on-demand (`.typed::<HeaderType>()`).

---

## 3. Extensibility (Plugins)

- **Tier 1 (WASM)**: In-process sandboxed execution (<1ms). Uses `wasmtime`. Plugins receive raw SIP messages parsed via `sipcore` in WASM.
- **Tier 2 (gRPC)**: Out-of-process event & control service via `pbx-proto`. Allows external modules written in Go, Python, C#, Node.js, etc.

---

## 4. Documentation Index

The project architecture plan is modularized under `docs/plan/`:
- `00-mission.md`: Mission & 5 pillars
- `01-workspace-structure.md`: Workspace structure & dependency graph
- `02-sipcore-design.md`: `sipcore` parser design & zero-copy model
- `03-sdpcore-design.md`: `sdpcore` parser & SDP negotiation
- `04-sipstack-design.md`: Transport, transactions & dialogs
- `05-distributed-architecture.md`: IPC protocols & WebRTC media routing
- `06-binary-roles.md`: B2BUA call bridge model
- `07-fault-recovery.md`: Shutdown & WAL snapshot recovery
- `08-plugin-system.md`: WASM & gRPC plugin system
- `09-storage.md`: Databases (SQLite, MongoDB) & config
- `10-rfc-matrix.md`: RFC matrix & crate mapping
- `11-implementation-plan.md`: 32-week roadmap
- `12-deployment-model.md`: Single-binary deployment & FS/Asterisk comparison
- `13-rest-api.md`: Axum REST API & WebSocket events
- `14-coding-standards.md`: English-only, modular file limits (<400 lines), clippy & fmt rules
