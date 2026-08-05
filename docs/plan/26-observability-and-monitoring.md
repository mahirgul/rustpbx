# 26 — Observability, Prometheus Metrics & OpenTelemetry Tracing

## 1. Executive Summary & Purpose

The Observability module (`bins/sip-engine/src/observability/`) equips RustPBX with enterprise-grade telemetry, real-time metrics, distributed tracing, and diagnostic packet capture.

---

## 2. Key Components

### A. Prometheus Metrics Endpoint (`GET /metrics`)
Exposes real-time system and telephony metrics:
- `rustpbx_active_calls`: Gauge of current active call bridges.
- `rustpbx_sip_messages_total`: Counter of incoming/outgoing SIP requests & responses by method/status.
- `rustpbx_rtp_jitter_milliseconds`: Histogram of media stream jitter.
- `rustpbx_rtp_packet_loss_ratio`: Gauge of stream packet loss.

### B. OpenTelemetry Distributed Tracing
- Traces individual calls end-to-end across processes: `webrtc-gateway` → `sip-engine` → `media-node` → `ai-agent`.
- Exported via gRPC OTLP to Jaeger, Tempo, or Datadog for sub-millisecond call sequence debugging.

### C. HEP/HOMER Packet Capture Integration
- Exports raw SIP text and RTCP quality reports to HOMER SIP Capture server via HEP (Horton Encapsulation Protocol v3).
- Enables visual call flow debugging and MOS (Mean Opinion Score) voice quality analytics.

---

## 3. Implementation Phase

Scheduled for **Phase 4 (Week 32)** during production hardening.
