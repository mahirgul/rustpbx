# 01 — Cargo Workspace Structure

## Directory Layout

```text
rustpbx/
├── Cargo.toml                          # Workspace root
├── _vendor/                            # Reference source code (not compiled, .gitignore'd)
│   ├── rsip/                           # rsip v0.4.0 reference (MIT)
│   └── rsipstack/                      # rsipstack v0.6.0 reference (MIT)
│
├── crates/
│   │
│   │  ┌──────────────────────────────────────────────────────────┐
│   │  │  LAYER 1: Pure Libraries (no tokio, no I/O, WASM-safe)  │
│   │  └──────────────────────────────────────────────────────────┘
│   │
│   ├── sipcore/                        # SIP message parser & types ONLY
│   │   ├── Cargo.toml                  # deps: nom, bytes, sipcore-derives
│   │   ├── sipcore-derives/            # Proc macro subcrate
│   │   │   ├── Cargo.toml
│   │   │   └── src/lib.rs
│   │   └── src/
│   │       ├── lib.rs                  # Public API & prelude
│   │       ├── parser/                 # nom-based SIP message parser
│   │       │   ├── mod.rs
│   │       │   ├── message.rs          # SipMessage, Request, Response tokenizers
│   │       │   ├── header.rs          # Header line tokenizer (with folding support)
│   │       │   └── uri.rs              # SIP URI parser
│   │       ├── types/                  # Core SIP types (AST)
│   │       │   ├── mod.rs
│   │       │   ├── method.rs           # SIP Method enum
│   │       │   ├── status_code.rs      # StatusCode enum (1xx-6xx)
│   │       │   ├── version.rs          # SIP/2.0
│   │       │   ├── uri.rs              # Uri, Scheme, HostWithPort, Params
│   │       │   ├── message.rs          # SipMessage, Request, Response structs
│   │       │   └── headers/            # Header system
│   │       │       ├── mod.rs          # Headers collection (Vec + HashMap index)
│   │       │       ├── header.rs       # Header enum (~50 variants + Other)
│   │       │       ├── typed/          # Structured typed headers (on-demand parse)
│   │       │       └── untyped/        # Fast Bytes-wrapped headers
│   │       ├── error.rs               # Unified error types
│   │       └── services/
│   │           └── digest.rs           # RFC 2617/7616 digest auth generator
│   │
│   ├── sdpcore/                        # SDP parser & media negotiation ONLY
│   │   ├── Cargo.toml                  # deps: nom, bytes (NO sipcore dependency)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs              # nom-based SDP parser
│   │       ├── types.rs               # SessionDescription, MediaDescription
│   │       ├── codec.rs               # Codec registry (Opus, G.711, G.722, etc.)
│   │       ├── ice.rs                 # ICE candidate parsing
│   │       └── negotiator.rs          # Offer/Answer model (RFC 3264)
│   │
│   │  ┌──────────────────────────────────────────────────────────┐
│   │  │  LAYER 2: Async Stack (tokio-dependent, network I/O)    │
│   │  └──────────────────────────────────────────────────────────┘
│   │
│   ├── sipstack/                       # SIP transport + transaction + dialog
│   │   ├── Cargo.toml                  # deps: sipcore, sdpcore, tokio, dashmap, rustls...
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── transport/              # Network transport layer
│   │       │   ├── mod.rs
│   │       │   ├── udp.rs
│   │       │   ├── tcp.rs
│   │       │   ├── tls.rs              # rustls-based
│   │       │   ├── websocket.rs        # tungstenite-based
│   │       │   ├── stream.rs           # TCP/TLS framing (Content-Length aware codec)
│   │       │   ├── connection.rs       # SipConnection enum
│   │       │   └── pool.rs             # DashMap<SipAddr, SipConnection> per-process pool
│   │       ├── transaction/            # RFC 3261 §17 transaction state machines
│   │       │   ├── mod.rs
│   │       │   ├── state_machine.rs    # ICT/NICT/IST/NIST states
│   │       │   ├── timer.rs            # Timers A-K (tokio::time based)
│   │       │   ├── key.rs              # Transaction matching (Via branch)
│   │       │   └── endpoint.rs         # Transaction user (TU) interface
│   │       ├── dialog/                 # RFC 3261 §12 dialog management
│   │       │   ├── mod.rs
│   │       │   ├── dialog.rs           # DialogId, DialogInner, route set
│   │       │   ├── invite_dialog.rs    # Single-role InviteDialog (one per leg)
│   │       │   ├── registration.rs     # REGISTER client/server
│   │       │   ├── subscription.rs     # RFC 6665 SUBSCRIBE/NOTIFY
│   │       │   └── authenticate.rs     # Digest auth challenge/response flow
│   │       ├── resolver/               # RFC 3263 DNS resolution
│   │       │   └── mod.rs              # NAPTR → SRV → A/AAAA chain (hickory-resolver)
│   │       └── traits/                 # Extension points
│   │           ├── mod.rs
│   │           ├── inspector.rs        # MessageInspector (pre-send / post-receive hooks)
│   │           ├── locator.rs          # TargetLocator (custom routing)
│   │           └── whitelist.rs        # TransportWhitelist (IP-level accept/reject)
│   │
│   │  ┌──────────────────────────────────────────────────────────┐
│   │  │  LAYER 3: Shared Definitions (protobuf, IPC contracts)  │
│   │  └──────────────────────────────────────────────────────────┘
│   │
│   └── pbx-proto/                      # Shared Protobuf/gRPC definitions
│       ├── Cargo.toml
│       ├── build.rs
│       └── proto/
│           ├── media_control.proto     # sip-engine ↔ media-node commands
│           ├── call_events.proto       # Real-time call state events
│           ├── plugin_api.proto        # Tier 2 plugin interface
│           └── gateway_media.proto     # webrtc-gateway ↔ sip-engine media coordination
│
│  ┌──────────────────────────────────────────────────────────────────┐
│  │  LAYER 4: Executable Binaries (independent processes)           │
│  └──────────────────────────────────────────────────────────────────┘
│
├── bins/
│   ├── sip-engine/                     # Core B2BUA Engine
│   ├── media-node/                     # RTP Processing & Relay
│   ├── webrtc-gateway/                 # WebRTC Edge Gateway
│   ├── web-admin/                      # Management UI Backend
│   ├── call-center/                    # ACD / Queue Service
│   └── voicemail/                      # Voicemail & STT Service
│
└── tests/
    ├── sip_compliance/                 # RFC 3261 conformance tests
    ├── integration/                    # Multi-binary integration tests
    └── fixtures/                       # Sample SIP messages & SDP bodies
```

## Dependency Graph (Unidirectional, No Cycles)

```text
                    ┌──────────┐     ┌──────────┐
                    │ sipcore  │     │ sdpcore  │
                    │ (parser  │     │ (parser  │
                    │  & types)│     │  & types)│
                    └────┬─────┘     └────┬─────┘
                         │                │
                         │   NO dependency between
                         │   sipcore ↔ sdpcore
                         │                │
                         ▼                ▼
                    ┌─────────────────────────┐
                    │       sipstack          │
                    │  (transport, transaction│
                    │   dialog — uses both)   │
                    └────────────┬────────────┘
                                │
                    ┌───────────┼───────────┐
                    ▼           ▼           ▼
              ┌──────────┐ ┌────────┐ ┌──────────────┐
              │sip-engine│ │media-  │ │webrtc-gateway│
              │          │ │node    │ │              │
              └──────────┘ └────────┘ └──────────────┘

  All bins also depend on: pbx-proto (gRPC definitions)
```
