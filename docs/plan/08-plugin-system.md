# 08 — Two-Tiered Plugin Architecture

## Tier 1: In-Process WASM Plugins

```text
┌─────────────────────────────────────────────────────────┐
│                    sip-engine process                     │
│                                                         │
│  ┌────────────────────────────────────────────────────┐ │
│  │              wasmtime sandbox                       │ │
│  │                                                    │ │
│  │  WASM Plugin has access to:                        │ │
│  │  ✅ sipcore (parse/modify SIP messages)            │ │
│  │  ✅ sdpcore (inspect SDP bodies)                   │ │
│  │  ❌ sipstack (no network, no tokio)                │ │
│  │  ❌ filesystem, sockets, system calls              │ │
│  │                                                    │ │
│  │  Host functions provided:                          │ │
│  │  - route_call(destination) → routing decision      │ │
│  │  - modify_header(name, value) → header change      │ │
│  │  - reject_call(status_code) → call rejection       │ │
│  │  - log(level, message) → structured logging        │ │
│  └────────────────────────────────────────────────────┘ │
│                                                         │
│  Latency: <1ms  |  Safety: memory-sandboxed             │
└─────────────────────────────────────────────────────────┘
```

**This is why `sipcore` must be IO-free:** It compiles to `wasm32-wasi`, enabling WASM plugins to parse and manipulate SIP messages without any network dependencies.

---

## Tier 2: Out-of-Process gRPC Plugins

* **Protocol:** gRPC (via `pbx-proto/plugin_api.proto`).
* **Timeout:** Configurable per-plugin (default: 100ms, max: 5000ms). On timeout, execute configurable fallback routing plan.
* **Language Support:** Go, Python, C#, Node.js, Java, Rust — any language supporting gRPC.
* **Use Cases:** CRM integration, call center queuing, AI speech recognition, billing systems.
