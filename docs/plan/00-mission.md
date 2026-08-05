# 00 — Mission & Vision

Replace legacy C/C++ monolithic PBX software like Asterisk and FreeSWITCH with a next-generation SIP PBX built with **100% Pure Rust**.

---

## 5 Core Pillars

### 🛡️ 1. 100% Pure Rust — Zero C/C++ Dependencies
No external C/C++ libraries required. Compiles directly via `cargo build --release` into a static binary ready for deployment on the target server. Zero dependencies on system libraries (libc, OpenSSL, libsrtp) — fully self-contained using `rustls`, `tokio`, `nom`, and project-internal crates.

### 🧱 2. Process Isolation — Each Component an Independent Executable
Split the system into independent executable files rather than a single monolith. Even if the WebRTC layer comes under attack or the voicemail module crashes, the **core SIP engine is guaranteed to remain running uninterrupted**. A process crash in one satellite service never affects the others.

### ⚡ 3. Two-Tiered Plugin Architecture
- **Tier 1 — WASM (In-Process):** Sandboxed micro-decisions executing inside the SIP engine at `<1ms` speed. Call routing, blacklist checks, header manipulation.
- **Tier 2 — gRPC (Out-of-Process):** External executable modules written in any language (Go, Python, C#, Node.js, Java). CRM integration, call center queues, AI speech recognition — all running as independent services.

### 📦 4. Single-Binary Deployment — Zero Configuration Confusion
Each component (`sip-engine`, `media-node`, `webrtc-gateway`) is statically linked into a single executable binary. Simply `scp` to the target server, adjust the TOML config file, and run. Docker is optional — not mandatory. No package manager dependencies (`apt install`, `yum`).

### 🏗️ 5. Three-Crate Core Architecture (Three-Crate SIP Foundation)
Project-internal SIP libraries are separated by concern:

| Crate | Layer | Characteristics |
|:---|:---|:---|
| `sipcore` | Parser & Types | Pure library — No I/O, no `tokio`, compiles to WASM |
| `sdpcore` | SDP Parser & Negotiation | Pure library — Independent of `sipcore`, compiles to WASM |
| `sipstack` | Transport, Transaction, Dialog | Async I/O — Uses `tokio`, `rustls`, `dashmap` |

Thanks to this separation:
- WASM plugins can use `sipcore` + `sdpcore` to parse SIP/SDP without network/tokio dependencies.
- Testing tools and log parsers can import `sipcore` without pulling heavy async dependencies.
- Zero circular dependency risk — strictly unidirectional: `sipstack` → `sipcore` + `sdpcore`.
