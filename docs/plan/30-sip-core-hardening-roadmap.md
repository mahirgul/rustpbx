# 30 — SIP Core Hardening Roadmap (Toward Routr-Class Robustness)

> **Status:** Planned (not implemented). Decided on 2026-11-07.
> **Decision:** We write **our own** RFC 3261 stack in `sipstack` (no third-party SIP stack).
> **Product target:** a robust, Routr-class SIP proxy / registrar / location service, plus a simple RTP relay. FreeSWITCH-class media (transcoding, conferencing, IVR) is a later, optional track delivered through modules ([31](31-modular-plugin-and-codec-api.md)), not a goal of the core.

---

## 1. Gap Analysis (2026-11-07)

| Area | Current state | Needed |
|---|---|---|
| Transaction layer | `TransactionState` is an enum only; no timers, no retransmission | ICT / NICT / IST / NIST state machines with Timers A–K (RFC 3261 §17) |
| Transports | UDP only | UDP, TCP, TLS (`rustls`), WS/WSS (RFC 7118) |
| Proxy behaviour | INVITE is copied to the callee; a fake `180` is sent to the caller; responses are not relayed | Stateful proxy: Via/branch handling, Record-Route, Max-Forwards, loose routing, response relay, ACK and CANCEL matching |
| BYE / CANCEL | Answered locally (`200`, `487`) without forwarding | Forwarded within dialog / matched transaction |
| Authentication | Nonce is not stored (replay possible); INVITE auth is skipped when the caller is not a known extension | Nonce store with expiry and `nc` tracking, `qop=auth`, SHA-256 (RFC 8760), reject unknown users |
| Location | One row per extension in SQLite | In-memory, multiple contacts per AOR, expiry, `q` values, OPTIONS keep-alive |
| Tests | ~3 unit tests | Unit, fuzz, torture (RFC 4475), SIPp scenarios in CI |

### Immediate security fixes (before any feature work)
1. `invite.rs`: when the caller user from `Authorization` is not a known extension, respond `403` instead of continuing.
2. Store issued nonces with a TTL and reject unknown or expired nonces.

---

## 2. Milestones

### M0 — Test & Quality Foundation
- GitHub Actions: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`.
- `cargo-fuzz` targets for `sipcore` message, header and URI parsers and `sdpcore`.
- RFC 4475 torture message corpus as parser tests.
- SIPp scenarios (REGISTER, basic call, CANCEL, BYE, auth challenge) running against `sip-engine` in CI.
- Close the two security issues above.

### M1 — RFC 3261 Core in `sipstack`
- Transaction layer: four state machines, Timers A–K, retransmission, transaction matching by branch (RFC 3261 §17.1.3 / §17.2.3).
- Transports: `Transport` trait; UDP, TCP (`tokio_util::codec` framing), TLS, WS/WSS; connection reuse (RFC 5923 optional).
- `rport` (RFC 3581), `received`, symmetric response routing.
- Dialog layer: dialog creation from 2xx / 1xx with To-tag, route set, CSeq tracking.
- Stateful proxy core: Record-Route, loose routing (`lr`), Max-Forwards, response relay, CANCEL forwarding, ACK for 2xx vs non-2xx.
- In-memory location service (`LocationStore` trait), digest auth with nonce store.
- Includes decoupling phases D1–D2 from [29](29-sip-admin-decoupling.md).

### M2 — Routr-Class Features
- Location service: multiple contacts per AOR, forking (parallel / sequential), `q` values.
- Outbound REGISTER to SIP providers; trunks, peers, domains.
- Routing: dialplan, least-cost routing, WASM routing plugins ([08](08-plugin-system.md), [31](31-modular-plugin-and-codec-api.md)).
- DNS NAPTR / SRV / A resolution and failover (RFC 3263).
- NAT and edge proxy mode, Path header (RFC 3327), Outbound (RFC 5626) keep-alives.
- Security: rate limiting, brute-force IP ban ([23](23-security-and-toll-fraud.md)).
- Multi-node: shared `LocationStore` backend (e.g. Redis) behind the same trait.
- Control API and `ConfigSource` (D3–D4 from [29](29-sip-admin-decoupling.md)).

### M3 — B2BUA & Media (optional, module-driven)
- Two-leg B2BUA with independent dialogs; re-INVITE / UPDATE (RFC 3311), PRACK (RFC 3262), REFER (RFC 3515), session timers (RFC 4028), hold.
- SDP rewriting and RTP relay through `media-node`; RFC 2833 / 4733 DTMF; recording.
- Codec API and module host ([31](31-modular-plugin-and-codec-api.md)).

### M4 — Production Readiness
- Graceful shutdown and state snapshot ([07](07-fault-recovery.md)).
- Prometheus metrics, HEP export ([26](26-observability-and-monitoring.md)).
- Load testing with SIPp (target: sustained hundreds of CPS per core, measured and published).
- Interoperability testing against Asterisk, Kamailio and real desk phones / softphones.

---

## 3. Rough Effort (experience-based estimate, not measured)

With 1–2 engineers: M0–M2 take several months; M3 roughly doubles that. Estimates will be revisited after M0 provides SIPp baselines.
