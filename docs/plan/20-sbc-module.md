# 20 — SBC (Session Border Controller) Module Architecture

## 1. Executive Summary & Purpose

The **SBC (Session Border Controller)** functionality is built as an in-process high-performance pipeline inside `sip-engine` (`bins/sip-engine/src/sbc/`).

Operating in-process rather than as a separate executable avoids cross-process serialization overhead, providing **zero-copy in-memory SIP packet transformation** before network transmission and upon network reception.

---

## 2. Core SBC Responsibilities

```text
Incoming SIP Packet ──► [Inbound SBC Pipeline] ──► Transaction/Dialog Layer
                                                          │
Outgoing SIP Packet ◄── [Outbound SBC Pipeline] ◄─────────┘
```

### Key Functions

1. **Topology Hiding**:
   - Strips internal private IP addresses (`10.x.x.x`, `192.168.x.x`) from `Via`, `Record-Route`, `Route`, and `Contact` headers.
   - Replaces private IPs with the public-facing SBC transport address.
2. **NAT Traversal (RFC 3581 `rport`)**:
   - Inspects `Via` header; appends `received` and `rport` parameters matching the actual source socket address.
   - Solves asymmetric NAT routing for remote SIP phones.
3. **Header Normalization & Rule Engine**:
   - Adds, modifies, or removes headers based on trunk requirements (e.g., `P-Asserted-Identity`, `Remote-Party-ID`, `X-Custom-Header`).
   - Strips revealing `User-Agent` or `Server` headers for security hardening.
4. **SDP Body Rewriting & Codec Filtering**:
   - Rewrites media connection IP addresses (`c=`) and ports (`m=`) to point to `media-node`.
   - Filters out unsupported or restricted audio/video codecs from SDP offer/answer bodies.

---

## 3. Module Layout inside `sip-engine`

```text
bins/sip-engine/src/sbc/
├── mod.rs                  # SBC Pipeline Manager
├── topology_hiding.rs     # IP masking in Via, Record-Route, Contact
├── header_rules.rs         # Rules for header modification/stripping
├── nat_traversal.rs        # RFC 3581 rport & received parameter handling
└── sdp_rewriter.rs         # SDP media IP & codec manipulation
```

---

## 4. Integration with `sipstack` MessageInspector Trait

The SBC pipeline plugs directly into `sipstack`'s zero-copy message pipeline:

```rust
use async_trait::async_trait;
use sipcore::types::SipMessage;
use std::net::SocketAddr;

pub struct SbcPipeline {
    topology_hiding: TopologyHiding,
    header_rules: HeaderRules,
    nat_traversal: NatTraversal,
}

#[async_trait]
impl MessageInspector for SbcPipeline {
    async fn on_receive(&self, msg: &mut SipMessage, src: &SocketAddr) -> Result<(), SbcError> {
        self.nat_traversal.process_inbound(msg, src)?;
        self.topology_hiding.mask_inbound(msg)?;
        Ok(())
    }

    async fn on_send(&self, msg: &mut SipMessage, dest: &SocketAddr) -> Result<(), SbcError> {
        self.header_rules.apply_outbound(msg)?;
        self.topology_hiding.mask_outbound(msg, dest)?;
        Ok(())
    }
}
```

---

## 5. Implementation Phase Schedule

Scheduled for **Phase 2 (Weeks 15-16)** alongside `sip-engine` B2BUA core development.
