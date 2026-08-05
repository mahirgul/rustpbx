# 23 — Intrusion Detection, Security & Toll Fraud Prevention

## 1. Executive Summary & Purpose

The Security module (`bins/sip-engine/src/security/`) protects RustPBX from brute-force REGISTER attacks, SIP scanning, denial of service (DoS), and unauthorized high-cost international call fraud (Toll Fraud).

---

## 2. Security Modules & Policies

### A. Rate Limiter & IP Ban Engine (`rate_limiter.rs`)
- Token bucket algorithm per IP address.
- Automatically bans offending IP addresses (temporary 1-hour ban or permanent ban) upon:
  - 5 failed REGISTER / Digest Auth attempts within 60 seconds.
  - Excessive SIP message rates (>200 req/sec from a single IP).

### B. Toll Fraud Protection (`toll_fraud.rs`)
- Evaluates outbound calls against destination rules and time-based limits.
- **Rule Engine**:
  - Restricts high-cost international prefix destinations unless explicitly enabled per extension.
  - Detects unusual burst call activity (e.g., 50 simultaneous outbound calls at 3 AM) and automatically freezes the trunk, triggering an emergency notification to the administrator.

### C. SIP Digest Authentication Hardening (`auth_guard.rs`)
- Enforces strict SHA-256 Digest Authentication (RFC 7616).
- Rejects weak passwords and enforces nonce freshness with automatic expiration to prevent replay attacks.

---

## 3. Implementation Phase

Scheduled for **Phase 4 (Weeks 29-30)** during production hardening.
