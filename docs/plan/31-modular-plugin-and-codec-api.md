# 31 — Modular Extension Points & Codec Module API

> **Status:** Planned (not implemented). Decided on 2026-11-07.
> **Requirement:** RustPBX must be modular. When a codec is missing or a new one is released, anyone must be able to write a module and plug it in. The same applies to other extension points (transports, storage, auth, routing).

This document extends the two-tier plugin model in [08](08-plugin-system.md).

---

## 1. Principles

1. **Signaling is codec-agnostic.** `sip-engine` never encodes or decodes audio. Codecs live only in `media-node`.
2. **Every extension point is a trait** in a small, stable API crate. The core depends only on traits, never on concrete modules.
3. **Three loading tiers**, chosen per use case:

| Tier | Mechanism | Hot-pluggable | Isolation | Use for |
|---|---|---|---|---|
| A | Rust crate implementing the trait, enabled via Cargo feature | No (rebuild) | Same process | First-party, performance-critical modules |
| B | WASM component (`wasmtime`, WIT interface) loaded from a directory | Yes | Sandboxed, cannot crash the host | Third-party codecs, routing logic |
| C | gRPC out-of-process service ([08](08-plugin-system.md) Tier 2) | Yes | Separate process | Heavy or non-Rust integrations (AI, billing, CRM) |

**Not supported:** native `.so` / `.dll` loading of Rust modules. Rust has no stable ABI and a faulty module would crash `media-node`. If ever required, it must go through a C ABI shim.

---

## 2. Codec Module API

### 2.1 Crates

```text
crates/
├── pbx-codec-api/        # traits + registry (no I/O, no tokio)
└── codecs/
    ├── g711/             # PCMU / PCMA reference module (Tier A)
    ├── g722/
    └── opus/             # optional, behind a feature
```

### 2.2 Traits (sketch)

```rust
/// Static description used for SDP negotiation.
pub struct CodecInfo {
    pub name: &'static str,          // e.g. "PCMU"
    pub clock_rate: u32,             // e.g. 8000
    pub channels: u8,
    pub default_payload_type: Option<u8>,
    pub fmtp: Option<String>,
    pub frame_ms: u16,               // e.g. 20
}

pub trait AudioCodec: Send + Sync {
    fn info(&self) -> &CodecInfo;
    fn new_encoder(&self, params: &CodecParams) -> Result<Box<dyn Encoder>, CodecError>;
    fn new_decoder(&self, params: &CodecParams) -> Result<Box<dyn Decoder>, CodecError>;
}

pub trait Encoder: Send {
    /// Encode one frame of 16-bit linear PCM into `out`; returns bytes written.
    fn encode(&mut self, pcm: &[i16], out: &mut [u8]) -> Result<usize, CodecError>;
}

pub trait Decoder: Send {
    /// Decode one payload into 16-bit linear PCM; returns samples written.
    fn decode(&mut self, payload: &[u8], pcm: &mut [i16]) -> Result<usize, CodecError>;
    /// Packet loss concealment for a missing frame.
    fn conceal(&mut self, pcm: &mut [i16]) -> Result<usize, CodecError>;
}

pub struct CodecRegistry { /* name/clock_rate -> Arc<dyn AudioCodec> */ }
```

### 2.3 WASM codec modules (Tier B)
- WIT interface `codec.wit` mirrors the traits above (`info`, `encoder-new`, `encode`, `decoder-new`, `decode`, `conceal`).
- `media-node` scans `plugins/codecs/*.wasm` at start-up and on a `ReloadModules` call, then registers each module in `CodecRegistry` through an adapter implementing `AudioCodec`.
- Per-call resource limits (fuel / epoch interruption, memory cap). A trapping module fails only the affected stream.
- Frame-based calls (typically 20 ms) keep host/guest crossing overhead small.

### 2.4 Negotiation flow
1. `media-node` exposes `ListCodecs` over gRPC (`pbx-proto`).
2. `sip-engine` caches the list and passes it to `sdpcore::negotiator`, replacing today's hard-coded codec names.
3. When both legs share a codec, media is relayed without decoding (pass-through). The codec module is used only for transcoding, recording, or playback.

---

## 3. Other Extension Points (`crates/pbx-plugin-api`)

| Trait | Default implementation | Alternatives |
|---|---|---|
| `Transport` | UDP, TCP, TLS, WS | QUIC (future) |
| `LocationStore` | In-memory `DashMap` | Redis (multi-node) |
| `ConfigSource` | SQLite (`pbx-store`) | TOML file, HTTP API |
| `Authenticator` | Digest over `ConfigSource` | LDAP, HTTP webhook |
| `CdrSink` | SQLite | File, Kafka, HTTP |
| Routing hook | Built-in dialplan | WASM routing plugin ([08](08-plugin-system.md)) |

---

## 4. Roadmap Placement (see [30](30-sip-core-hardening-roadmap.md))

- **M1:** `pbx-plugin-api` with `Transport`, `LocationStore`, `ConfigSource`, `Authenticator`.
- **M2:** WASM routing plugin host; Redis `LocationStore`.
- **M3:** `pbx-codec-api`, G.711 reference module, WASM codec host, `ListCodecs` RPC.
- Every module API is versioned (semver on the API crate, WIT package version for WASM).
