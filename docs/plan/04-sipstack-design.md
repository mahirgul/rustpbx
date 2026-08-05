# 04 — `sipstack` Design — Async SIP Stack

## Purpose

Tokio-based transport, transaction state machines, and dialog management.

**Dependencies:** `sipcore`, `sdpcore`, `tokio`, `dashmap`, `rustls`, `tokio-tungstenite`, `hickory-resolver`

---

## Transport Layer

Each process maintains its own independent connection pool — `DashMap` is process-local and never shared across OS processes:

```text
┌─────────────────────────────────────────────────────────────┐
│  TransportLayer (per-process, inside sipstack)               │
│                                                             │
│  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────────┐       │
│  │  UDP   │  │  TCP   │  │  TLS   │  │ WebSocket  │       │
│  └───┬────┘  └───┬────┘  └───┬────┘  └─────┬──────┘       │
│      └─────┬─────┴─────┬─────┴─────┬────────┘              │
│            ▼           ▼           ▼                        │
│      SipConnection (enum wrapper)                           │
│            │                                                │
│            ▼                                                │
│  DashMap<SipAddr, SipConnection>  ← PROCESS-LOCAL ONLY      │
│            │                                                │
│            ▼                                                │
│  TransportEvent { message, source_addr, transport }         │
└─────────────────────────────────────────────────────────────┘
```

---

## Transaction Layer (RFC 3261 §17)

| Transaction | States | Key Timers |
|:---|:---|:---|
| **Client INVITE (ICT)** | Calling → Proceeding → Completed → Terminated | A (retransmit), B (timeout=32s), D (wait) |
| **Client Non-INVITE (NICT)** | Trying → Proceeding → Completed → Terminated | E (retransmit), F (timeout=32s), K (wait) |
| **Server INVITE (IST)** | Trying → Proceeding → Completed → Confirmed → Terminated | G (retransmit), H (ACK wait), I (confirmed) |
| **Server Non-INVITE (NIST)** | Trying → Proceeding → Completed → Terminated | J (wait) |

**RFC 6026 Compliance:** 2xx responses to INVITE bypass the transaction layer — ACK for 2xx is handled directly at the Dialog/TU layer.

---

## Dialog Layer (RFC 3261 §12)

```rust
pub struct DialogId {
    pub call_id: String,
    pub local_tag: String,
    pub remote_tag: String,
}

/// Each InviteDialog represents ONE role (UAC or UAS) in ONE dialog.
/// A B2BUA creates TWO InviteDialogs per call — one per leg.
pub struct InviteDialog {
    inner: Arc<DialogInner>,
}

struct DialogInner {
    id: DialogId,
    role: DialogRole,                  // UAC or UAS (fixed at creation)
    state: AtomicU8,                   // Early / Confirmed / Terminated
    local_cseq: AtomicU32,
    remote_cseq: AtomicU32,
    route_set: RwLock<Vec<Uri>>,       // Derived from Record-Route
    remote_target: RwLock<Uri>,        // Contact header URI
}
```

---

## Extension Traits

```rust
/// Inspect/modify SIP messages before send and after receive
#[async_trait]
pub trait MessageInspector: Send + Sync {
    async fn on_send(&self, msg: &mut SipMessage, dest: &SipAddr) -> Result<()>;
    async fn on_receive(&self, msg: &mut SipMessage, src: &SipAddr) -> Result<()>;
}

/// Custom routing for outbound message delivery
#[async_trait]
pub trait TargetLocator: Send + Sync {
    async fn locate(&self, msg: &SipMessage) -> Result<Vec<SipAddr>>;
}

/// IP-level accept/reject for incoming connections
#[async_trait]
pub trait TransportWhitelist: Send + Sync {
    async fn is_allowed(&self, addr: &SocketAddr) -> bool;
}
```
