# 12 — Deployment Model & Competitive Comparison

## Single Binary Principle

```text
cargo build --release
     │
     ▼
target/release/
├── sip-engine          # ~15-25 MB static binary (with musl)
├── media-node          # ~8-12 MB
├── webrtc-gateway      # ~10-15 MB
├── web-admin           # ~8-10 MB
├── call-center         # ~5-8 MB
└── voicemail           # ~5-8 MB
```

Each binary:
- **Statically linked** — Zero dependencies on glibc/musl, OpenSSL, libsrtp.
- **Cross-compilable** — Target `x86_64-unknown-linux-musl` runs on any Linux distribution.
- **Zero runtime** — No JVM, Python interpreter, or Node.js required.
- **Single file** — Deploy via `scp sip-engine user@server:/opt/rustpbx/`.

---

## Minimal Single-Server Topology

```text
/opt/rustpbx/
├── bin/
│   ├── sip-engine
│   ├── media-node
│   └── webrtc-gateway          # Optional (if WebRTC isn't needed)
├── config/
│   ├── sip-engine.toml         # SIP settings, trunks, dialplan
│   ├── media-node.toml         # RTP port range, codec priorities
│   └── webrtc-gateway.toml     # STUN/TURN, TLS certificates
├── data/
│   ├── rustpbx.db              # SQLite — config + dialplan + recovery snapshots
│   └── wasm-plugins/           # .wasm files (Tier 1 plugins)
└── logs/
```

---

## Distributed Multi-Server Topology

```text
┌─────────────────────────┐     ┌─────────────────────────┐
│    Server A (Signaling) │     │    Server B (Media)     │
│                          │     │                          │
│  ┌──────────────────┐   │     │  ┌──────────────────┐   │
│  │   sip-engine     │◄──gRPC───►│   media-node     │   │
│  └──────────────────┘   │     │  └──────────────────┘   │
│  ┌──────────────────┐   │     │  ┌──────────────────┐   │
│  │   web-admin      │   │     │  │   media-node (2) │   │
│  └──────────────────┘   │     │  └──────────────────┘   │
└─────────────────────────┘     └─────────────────────────┘

┌─────────────────────────┐
│    Server C (Edge)      │
│                          │
│  ┌──────────────────┐   │
│  │ webrtc-gateway   │◄──gRPC──► Server A
│  └──────────────────┘   │
└─────────────────────────┘
```

---

## Asterisk / FreeSWITCH Comparison

| Feature | Asterisk | FreeSWITCH | **RustPBX** |
|:---|:---|:---|:---|
| **Language** | C | C | **Rust** |
| **Memory Safety** | ❌ Manual malloc/free | ❌ Manual | ✅ Ownership system |
| **Crash Impact** | 💀 Whole system crashes | 💀 Whole system crashes | 🛡️ Only crashed process |
| **Build Dependencies** | ~40 packages | ~30 packages | **0** (only `cargo`) |
| **Deployment** | `make install` + package manager | `make install` + package manager | **`scp` + run** |
| **Plugin System** | Dialplan + AGI (process fork) | Lua/JS (in-process, unsafe) | **WASM (sandboxed) + gRPC** |
| **Plugin Languages** | C, AGI (any), AMI | C, Lua, JS, Python (ESL) | **WASM: Rust/C/Go + gRPC: Any** |
| **Plugin Isolation** | ❌ C module crash kills Asterisk | ❌ Lua error kills FreeSWITCH | ✅ WASM sandboxed + process isolation |
| **WebRTC** | Limited | Native (mod_verto) | **Separate process (security isolation)** |
| **Configuration** | `.conf` (complex) | XML (verbose) | **TOML (simple, type-safe)** |
| **Concurrent Calls** | ~1000 | ~3000 | **5000+ (tokio async, lock-free)** |
| **Codec Transcoding** | ✅ (C libraries) | ✅ (C libraries) | ✅ (Pure Rust) |
| **Community Maturity** | ⭐⭐⭐⭐⭐ (20+ years) | ⭐⭐⭐⭐ (15+ years) | ⭐ (New project) |
