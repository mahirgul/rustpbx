# 19 — Unified Web Admin Service & Complete PBX Management

## 1. Executive Summary & Single-Pane-of-Glass Vision

The **`web-admin`** binary (`bins/web-admin/`) is the central, unified administration dashboard and REST API gateway for RustPBX. It provides complete control over **every single PBX module and service** from a single web interface.

```text
                               ┌─────────────────────────────────────────┐
                               │            web-admin (.exe)             │
                               │        (Axum REST API + Dashboard)      │
                               └────────────────────┬────────────────────┘
                                                    │
     ┌──────────────────┬──────────────────┬────────┴─────────┬──────────────────┬──────────────────┐
     ▼                  ▼                  ▼                  ▼                  ▼                  ▼
┌──────────┐      ┌──────────┐      ┌──────────┐      ┌──────────┐      ┌──────────┐      ┌──────────┐
│  Core    │      │ Queue /  │      │ Security │      │ Time &   │      │ Conference│     │ Metrics &│
│  PBX     │      │ ACD      │      │ Fraud    │      │ Calendar │      │ & Media  │     │ Tracing  │
└──────────┘      └──────────┘      └──────────┘      └──────────┘      └──────────┘      └──────────┘
```

---

## 2. Complete Web Admin Management Modules

`web-admin` exposes intuitive REST endpoints and UI dashboards for all 26 architecture components:

| Dashboard Section | Managed Sub-Systems & Features |
|:---|:---|
| **Core PBX** | Extensions (SIP AORs, passwords), Trunks (PSTN/SIP providers), Active Call Bridges, Disconnect/Transfer controls |
| **Call Center / ACD** | Queue creation, Routing strategy assignment (`ring-all`, `round-robin`), Agent login/logout management, Real-time queue wait metrics, Callback requests ([#21](21-call-center-queues.md)) |
| **IVR Visual Builder** | Drag-and-drop IVR menu flow creation, Prompt uploader, DB query action builder, HTTP Webhook rules ([#16](16-ivr-and-voicemail.md)) |
| **Voicemail Admin** | Mailbox provisioning, Storage quotas, Email SMTP setup, Audio playback, Voicemail-to-Text transcriptions ([#16](16-ivr-and-voicemail.md)) |
| **AI & Speech** | Real-time AI agent prompts, Voice selection (ElevenLabs/Piper), STT/TTS engine parameters ([#17](17-ai-integration.md), [#18](18-tts-service.md)) |
| **Security & Fraud** | IP ban blacklist/whitelist, Brute-force rate limits, Toll fraud international destination rules, Security alert logs ([#23](23-security-and-toll-fraud.md)) |
| **Time & Calendars** | Business hours rules, Holiday override calendar management, Manual override toggles (`Forced Open / Closed`) ([#24](24-time-based-routing.md)) |
| **Conferences** | Conference room creation, PIN codes, Moderator controls (Mute/Unmute participants), Active speaker visualization ([#22](22-conference-bridge.md)) |
| **Fax over IP (T.38)** | Virtual Fax extension management, Inbound Fax-to-Email PDF settings, Outbound Email-to-Fax logs ([#25](25-fax-over-ip.md)) |
| **Observability** | Prometheus metric graphs, OpenTelemetry trace timeline viewer, Live HOMER SIP packet capture debugger ([#26](26-observability-and-monitoring.md)) |

---

## 3. Communication Protocols

- **Control Commands**: `web-admin` communicates with `sip-engine`, `media-node`, `call-center`, and `voicemail` via high-speed gRPC.
- **Config Persistence**: Writes changes directly to SQLite (WAL Mode), triggering instant atomic in-memory cache updates in `sip-engine`.
- **Live UI Updates**: Uses WebSockets (`/api/v1/events`) to stream real-time call states, active agent statuses, and system metrics directly to the browser.
