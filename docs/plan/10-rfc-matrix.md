# 10 — RFC Coverage Matrix & Tech Stack

## RFC Coverage Matrix

| RFC | Description | Crate / Location | Phase |
|:---|:---|:---|:---|
| 3261 | Core SIP | `sipcore` (parser), `sipstack` (transport/txn/dialog) | 🔴 Phase 1 |
| 3262 | PRACK (Reliable 1xx) | `sipstack/transaction` + `sipstack/dialog` | 🟡 Phase 3 |
| 3263 | DNS SRV/NAPTR | `sipstack/resolver` | 🟡 Phase 2 |
| 3264 | SDP Offer/Answer | `sdpcore/negotiator` | 🔴 Phase 1 |
| 3311 | UPDATE Method | `sipstack/dialog` | 🟡 Phase 3 |
| 3515 | REFER (Transfer) | `sip-engine/b2bua` | 🔴 Phase 2 |
| 3581 | rport (NAT) | `sipstack/transport` | 🔴 Phase 1 |
| 3891 | Replaces (Attended Transfer) | `sip-engine/b2bua` | 🟡 Phase 3 |
| 4566/8866 | SDP | `sdpcore` | 🔴 Phase 1 |
| 6026 | 2xx INVITE Fix | `sipstack/transaction` | 🔴 Phase 1 |
| 6665 | Event Framework | `sipstack/dialog/subscription` | 🟡 Phase 3 |

---

## Tech Stack Summary

| Category | Technology |
|:---|:---|
| **Language** | Rust (2021 edition) |
| **Async Runtime** | `tokio` |
| **SIP Parser & Types** | `sipcore` (internal, nom-based, WASM-compatible) |
| **SDP Parser & Negotiation** | `sdpcore` (internal, nom-based, WASM-compatible) |
| **SIP Async Stack** | `sipstack` (internal, tokio-based transport/txn/dialog) |
| **WASM Runtime** | `wasmtime` |
| **gRPC & Protobuf** | `tonic` / `prost` |
| **Web Framework & REST** | `axum` |
| **WebRTC Stack** | `webrtc-rs` |
| **TLS** | `rustls` / `tokio-rustls` |
| **WebSocket** | `tokio-tungstenite` |
| **DNS Resolution** | `hickory-resolver` |
| **Local Database** | `sqlx` (SQLite, WAL mode) |
| **CDR Store** | `mongodb` (async driver) |
| **Concurrent Maps** | `dashmap` |
| **Buffer Management** | `bytes` |
