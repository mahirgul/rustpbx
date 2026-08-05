# 06 — Binary Roles & B2BUA Call Bridge Model

## `sip-engine` (Core B2BUA Engine)

* **Role:** SIP signaling (RFC 3261), dialog & transaction management, B2BUA call bridging.
* **Uses:** `sipstack` for all SIP I/O, `sdpcore` for SDP manipulation.

### B2BUA Call Bridge Model

```rust
/// Each active call has exactly one CallBridge
struct CallBridge {
    id: CallId,
    
    /// Leg A: Incoming side — sip-engine acts as UAS
    leg_a: CallLeg,           // wraps InviteDialog with role=UAS
    
    /// Leg B: Outgoing side — sip-engine acts as UAC
    leg_b: CallLeg,           // wraps InviteDialog with role=UAC
    
    state: CallBridgeState,   // Ringing, Connected, OnHold, Transferring, Terminated
    
    /// Media coordination context
    media: MediaSession,      // media-node port allocations, SRTP keys
    
    /// Timestamps for CDR generation
    started_at: Instant,
    answered_at: Option<Instant>,
}

/// Process-local concurrent call tracking
struct CallManager {
    calls: DashMap<CallId, Arc<CallBridge>>,  // PROCESS-LOCAL ONLY
}
```

* **Guarantees:** Non-blocking async I/O. **Zero synchronous disk or database I/O** during active calls.
* **Media Policy:** Never handles RTP payload. Only negotiates SDP and orchestrates `media-node`.

---

## `webrtc-gateway` (Edge Border Element)
* Terminates WebRTC sessions (WSS, DTLS-SRTP, ICE/STUN/TURN).
* Communicates with `sip-engine` via gRPC (not raw SIP) for signaling.
* Converts WebRTC SDP (ICE, DTLS fingerprints, BUNDLE) → standard SDP.
* After media setup, routes decrypted RTP directly to `media-node`.

---

## `media-node` (RTP Processing & Relay)
* RTP/SRTP forwarding, Jitter Buffer, Audio Transcoding.
* Controlled by `sip-engine` via gRPC.
* Monitors `sip-engine` heartbeat — if lost, releases allocated ports after timeout.

---

## Service Binaries (`web-admin`, `call-center`, `voicemail`)
* Standalone microservices, communicate with `sip-engine` via gRPC events/commands.
