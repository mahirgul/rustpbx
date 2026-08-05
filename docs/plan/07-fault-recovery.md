# 07 — Graceful Shutdown & Crash Recovery

## Graceful Shutdown (SIGTERM / SIGINT)

```text
SIGTERM received by sip-engine
       │
       ▼
┌──────────────────────────┐
│ 1. Stop accepting new    │  Reject new INVITEs with 503 Service Unavailable
│    SIP connections       │
└──────────┬───────────────┘
           ▼
┌──────────────────────────┐
│ 2. Drain active calls    │  For each CallBridge:
│    (configurable timeout │    - Send BYE on both legs
│     default: 30s)        │    - Wait for 200 OK or timeout
└──────────┬───────────────┘
           ▼
┌──────────────────────────┐
│ 3. Release media         │  gRPC: ReleaseAll to media-node
│    resources             │
└──────────┬───────────────┘
           ▼
┌──────────────────────────┐
│ 4. Flush CDR buffer      │  Push remaining CDRs to MongoDB
│    to storage            │
└──────────┬───────────────┘
           ▼
         EXIT 0
```

---

## Crash Recovery

**Problem:** If `sip-engine` crashes, all in-memory call states are lost. Remote endpoints will timeout after 32s (Timer B) with dangling media sessions.

**Solution:** Periodic minimal state snapshots:

```rust
/// Minimal snapshot — just enough to send cleanup BYEs on restart
#[derive(Serialize, Deserialize)]
struct CallSnapshot {
    call_id: String,
    leg_a_dialog: DialogSnapshot,  // remote Contact, From/To tags, CSeq
    leg_b_dialog: DialogSnapshot,
    media_node_session_id: String,
    timestamp: SystemTime,
}

/// Written to SQLite (WAL mode) every 5 seconds, batched
/// On restart: load snapshots, send BYE to both legs, release media-node resources
```

---

## Satellite Process Recovery

- `media-node` and `webrtc-gateway` send heartbeat pings to `sip-engine` via gRPC every 5 seconds.
- If 3 consecutive heartbeats fail (15s):
  1. Stop accepting new sessions.
  2. Release all allocated media ports.
  3. Attempt to reconnect to `sip-engine`.
