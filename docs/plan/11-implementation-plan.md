# 11 — Phased Implementation Plan (32 Weeks)

## Engineering Discipline & Code Quality Policy

Across all phases, the following rules are strictly enforced:
- **Strict File Modularity**: No file shall exceed 300-400 lines of code. Split into granular submodules.
- **Mandatory Quality Gate**: Every phase build must pass `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- **100% English**: Code, comments, docs, error strings, and log messages must be in English.

---

## Phase 1A: Core Parser Libraries (Weeks 1-6)
> `sipcore` + `sdpcore` — pure zero-I/O libraries, full unit testing, WASM-compatible

1. **Weeks 1-2:** `sipcore/parser` — nom tokenizers, message/header/URI parsing, header line folding support.
2. **Weeks 3-4:** `sipcore/types` — AST types, `Bytes`-based header system, proc macros, digest auth generator.
3. **Weeks 5-6:** `sdpcore` — SDP parser, types, basic offer/answer negotiator (RFC 3264).
4. **Week 6 Validation:** Unit test suite, fuzzing with `cargo-fuzz`, and WASM target compilation (`wasm32-wasi`).

---

## Phase 1B: Async Stack (Weeks 7-12)
> `sipstack` — transport + transaction + dialog layers

1. **Weeks 7-8:** `sipstack/transport` — UDP transport, `SipConnection` abstraction, per-process `DashMap` pool.
2. **Weeks 9-10:** `sipstack/transaction` — ICT, NICT, IST, NIST transaction state machines (RFC 3261 §17), Timers A-K.
3. **Weeks 11-12:** `sipstack/transport` & `resolver` — TCP stream framing (`tokio_util::codec`), TLS via `rustls`, DNS NAPTR/SRV lookup via `hickory-resolver`.
4. **Week 12 Validation:** Conformance integration tests using `SIPp` scripts (INVITE/200 OK/ACK/BYE flows).

---

## Phase 2: B2BUA Engine & REST Control (Weeks 13-18)
> `sip-engine`, `media-node`, and Axum REST API — functional call bridging with media relay

1. **Weeks 13-14:** `sipstack/dialog` — `InviteDialog`, registration handling, route set management.
2. **Weeks 15-16:** `sip-engine/b2bua` & REST API (`api/rest.rs`) — `CallLeg`, `CallBridge`, SDP rewriter, `CallManager`, REST originate/hangup endpoints, authentication middleware.
3. **Weeks 17-18:** `media-node` — RTP/SRTP relay, gRPC control protocol, jitter buffer.
4. **Week 18 Validation:** End-to-end integration test (`pbx-proto` gRPC coordination: SIP Phone A → PBX → SIP Phone B).

---

## Phase 3: PBX Features, WASM & WebRTC (Weeks 19-26)
> Call transfers, event framework, Tier 1 WASM plugins, WebRTC edge gateway

1. **Weeks 19-20:** RFC 3515 REFER & RFC 3891 Replaces — Blind & Attended call transfer mechanisms.
2. **Weeks 21-22:** RFC 6665 Event Framework — BLF (Busy Lamp Field), MWI (Message Waiting Indicator), subscription state machines.
3. **Weeks 23-24:** Tier 1 WASM Plugin System — `wasmtime` sandbox integration (`sipcore` running inside WASM for <1ms routing/filter decisions).
4. **Weeks 25-26:** `webrtc-gateway` — WebRTC (WSS, DTLS-SRTP, ICE) ↔ SIP signaling & media path coordination.

---

## Phase 4: Hardening, Production & Tier 2 Plugins (Weeks 27-32)
> High availability, security, admin dashboard, gRPC plugins

1. **Weeks 27-28:** Fault tolerance — Graceful SIGTERM draining, SQLite WAL call state snapshots, heartbeat liveness monitoring.
2. **Weeks 29-30:** Security & Protection — SIP digest authentication, rate limiting (token bucket), toll fraud detection rules.
3. **Week 31:** Administration & Tier 2 Plugins — `web-admin` (REST API + management UI), Tier 2 gRPC plugin API (`call-center` queue service).
4. **Week 32:** Observability & Benchmarks — Prometheus metrics (`/metrics`), OpenTelemetry tracing, high-concurrency SIPp benchmarking (5000+ calls target), CI/CD pipelines, complete documentation.
