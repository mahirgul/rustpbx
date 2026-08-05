# 13 — Call Control with REST API

## Why REST?

| Channel | Usage | Target Audience |
|:---|:---|:---|
| **gRPC** | Internal high-performance IPC (`media-node`, `webrtc-gateway`) | System components |
| **REST** | External call control, integration | Web interface, CRM, mobile app, 3rd party |
| **WebSocket** | Real-time event stream (CallStarted, CallEnded...) | Dashboard, monitoring |

gRPC is powerful, but it is not easily accessible for every client (especially browsers, curl, Postman, or simple scripts). REST API allows **anyone who can send HTTP requests in any language** to control the PBX.

---

## Module Location

```text
bins/sip-engine/src/
├── api/
│   ├── mod.rs              # API router aggregation
│   ├── rest.rs              # Axum REST router (extended)
│   ├── grpc.rs              # tonic gRPC endpoints (internal IPC)
│   ├── websocket.rs         # Real-time event stream
│   ├── auth.rs              # API key / JWT authentication middleware
│   └── models.rs            # REST request/response JSON models
```

`rest.rs` runs on a single HTTP server using Axum. gRPC and WebSocket share the same port (Axum + Tonic multiplexing).

---

## API Endpoint Design

### Call Management

```text
POST   /api/v1/calls                    Initiate a new call (originate)
GET    /api/v1/calls                    List active calls
GET    /api/v1/calls/{call_id}          Get call details
DELETE /api/v1/calls/{call_id}          Terminate call (BYE)
POST   /api/v1/calls/{call_id}/hold     Place call on hold
POST   /api/v1/calls/{call_id}/unhold   Resume call from hold
POST   /api/v1/calls/{call_id}/mute     Mute microphone
POST   /api/v1/calls/{call_id}/unmute   Unmute microphone
POST   /api/v1/calls/{call_id}/transfer Call transfer (blind/attended)
POST   /api/v1/calls/{call_id}/dtmf     Send DTMF tone
```

### Registration & Trunk Management

```text
GET    /api/v1/registrations            List active SIP registrations
GET    /api/v1/trunks                   List trunks
GET    /api/v1/trunks/{trunk_id}/status Trunk status (registered/failed)
```

### System

```text
GET    /api/v1/system/health            Health check
GET    /api/v1/system/metrics           Prometheus metrics
GET    /api/v1/system/info              Version, uptime, active call count
POST   /api/v1/system/reload            Reload configuration
```

### WebSocket Event Stream

```text
GET    /api/v1/events                   WebSocket upgrade → real-time events
```

---

## Request/Response Examples

### Initiate New Call (Originate)

```http
POST /api/v1/calls
Content-Type: application/json
Authorization: Bearer <api-key>

{
  "from": "sip:100@pbx.local",
  "to": "sip:200@pbx.local",
  "trunk": "pstn-trunk-1",           // optional — outbound call via trunk
  "caller_id": "+905551234567",       // optional — caller ID
  "timeout_secs": 30,                 // optional — ring timeout
  "variables": {                      // optional — passed to WASM plugins
    "crm_ticket_id": "TK-12345",
    "priority": "high"
  }
}
```

```http
HTTP/1.1 201 Created
Content-Type: application/json

{
  "call_id": "550e8400-e29b-41d4-a716-446655440000",
  "state": "initiating",
  "from": "sip:100@pbx.local",
  "to": "sip:200@pbx.local",
  "created_at": "2026-08-05T15:45:00Z"
}
```

### List Active Calls

```http
GET /api/v1/calls
Authorization: Bearer <api-key>
```

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "total": 3,
  "calls": [
    {
      "call_id": "550e8400-...",
      "state": "connected",
      "from": "sip:100@pbx.local",
      "to": "sip:200@pbx.local",
      "direction": "inbound",
      "duration_secs": 142,
      "started_at": "2026-08-05T15:40:00Z",
      "answered_at": "2026-08-05T15:40:05Z"
    },
    ...
  ]
}
```

### Call Transfer

```http
POST /api/v1/calls/550e8400-.../transfer
Content-Type: application/json
Authorization: Bearer <api-key>

{
  "type": "blind",                    // "blind" or "attended"
  "target": "sip:300@pbx.local",
  "referred_by": "sip:100@pbx.local"  // optional
}
```

```http
HTTP/1.1 202 Accepted
Content-Type: application/json

{
  "call_id": "550e8400-...",
  "state": "transferring",
  "transfer_target": "sip:300@pbx.local"
}
```

### Send DTMF

```http
POST /api/v1/calls/550e8400-.../dtmf
Content-Type: application/json
Authorization: Bearer <api-key>

{
  "digits": "1234#",
  "duration_ms": 160                   // optional — duration per tone
}
```

### WebSocket Event Stream

```javascript
// Browser / Node.js client
const ws = new WebSocket("wss://pbx.example.com/api/v1/events?token=<api-key>");

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log(data);
};

// Incoming event examples:
// { "event": "call.started",   "call_id": "550e...", "from": "100", "to": "200", "timestamp": "..." }
// { "event": "call.ringing",   "call_id": "550e...", "timestamp": "..." }
// { "event": "call.answered",  "call_id": "550e...", "duration_secs": 0, "timestamp": "..." }
// { "event": "call.ended",     "call_id": "550e...", "duration_secs": 142, "reason": "bye", "timestamp": "..." }
// { "event": "call.dtmf",      "call_id": "550e...", "digit": "5", "timestamp": "..." }
// { "event": "reg.added",      "aor": "sip:100@pbx.local", "contact": "...", "timestamp": "..." }
// { "event": "trunk.down",     "trunk_id": "pstn-1", "reason": "timeout", "timestamp": "..." }
```

---

## Rust Implementation Outline

```rust
// bins/sip-engine/src/api/rest.rs

use axum::{
    Router, Json,
    extract::{Path, State, Query},
    routing::{get, post, delete},
    middleware,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Shared application state
pub struct AppState {
    pub call_manager: Arc<CallManager>,
    pub config: Arc<Config>,
}

/// Create REST router
pub fn rest_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Call management
        .route("/api/v1/calls",              post(originate_call))
        .route("/api/v1/calls",              get(list_calls))
        .route("/api/v1/calls/:call_id",     get(get_call))
        .route("/api/v1/calls/:call_id",     delete(hangup_call))
        .route("/api/v1/calls/:call_id/hold",     post(hold_call))
        .route("/api/v1/calls/:call_id/unhold",   post(unhold_call))
        .route("/api/v1/calls/:call_id/mute",     post(mute_call))
        .route("/api/v1/calls/:call_id/unmute",   post(unmute_call))
        .route("/api/v1/calls/:call_id/transfer", post(transfer_call))
        .route("/api/v1/calls/:call_id/dtmf",     post(send_dtmf))
        // Registrations & trunks
        .route("/api/v1/registrations",     get(list_registrations))
        .route("/api/v1/trunks",            get(list_trunks))
        .route("/api/v1/trunks/:trunk_id/status", get(trunk_status))
        // System
        .route("/api/v1/system/health",     get(health_check))
        .route("/api/v1/system/metrics",    get(prometheus_metrics))
        .route("/api/v1/system/info",       get(system_info))
        .route("/api/v1/system/reload",     post(reload_config))
        // WebSocket events
        .route("/api/v1/events",            get(ws_events))
        // Auth middleware
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

/// POST /api/v1/calls — Initiate a new call
async fn originate_call(
    State(state): State<Arc<AppState>>,
    Json(req): Json<OriginateRequest>,
) -> Result<Json<CallResponse>, ApiError> {
    // 1. Determine destination using dialplan/routing
    // 2. Create CallBridge (Leg A + Leg B)
    // 3. Send INVITE on Leg B
    // 4. Return call_id
    let call = state.call_manager.originate(req).await?;
    Ok(Json(call.into()))
}

/// DELETE /api/v1/calls/:call_id — Terminate call
async fn hangup_call(
    State(state): State<Arc<AppState>>,
    Path(call_id): Path<String>,
) -> Result<Json<CallResponse>, ApiError> {
    // Find CallBridge → send BYE to both legs
    let call = state.call_manager.hangup(&call_id).await?;
    Ok(Json(call.into()))
}

/// POST /api/v1/calls/:call_id/transfer — Call transfer
async fn transfer_call(
    State(state): State<Arc<AppState>>,
    Path(call_id): Path<String>,
    Json(req): Json<TransferRequest>,
) -> Result<Json<CallResponse>, ApiError> {
    // Blind: send REFER (RFC 3515)
    // Attended: create new leg, bind with Replaces (RFC 3891)
    let call = state.call_manager.transfer(&call_id, req).await?;
    Ok(Json(call.into()))
}
```

---

## Security

| Layer | Mechanism |
|:---|:---|
| **Authentication** | API Key (header: `Authorization: Bearer <key>`) or JWT token |
| **Authorization** | Role-based: `admin` (all operations), `operator` (call control), `viewer` (read-only) |
| **Rate limiting** | Token bucket per API key (default: 100 req/s) |
| **TLS** | HTTPS mandatory (in production), self-signed certificate in dev environment |
| **CORS** | Configurable origin list for `web-admin` frontend |
| **Audit log** | Every call control operation is logged (who, what, when) |

---

## Architectural Flow: REST → SIP

```text
  HTTP Client (curl, browser, CRM)
       │
       │  POST /api/v1/calls  { from: "100", to: "200" }
       ▼
  ┌─────────────────────────────────────────┐
  │  sip-engine / api / rest.rs             │
  │  (Axum HTTP handler)                    │
  │                                         │
  │  1. Auth middleware → validate API key  │
  │  2. JSON deserialize → OriginateRequest │
  │  3. Validate input                      │
  └────────────────┬────────────────────────┘
                   │
                   ▼
  ┌─────────────────────────────────────────┐
  │  sip-engine / b2bua / call_manager.rs   │
  │                                         │
  │  4. Call WASM plugin (Tier 1)           │
  │     → routing decision, add headers     │
  │  5. Create CallBridge                   │
  │     → leg_a (UAS) + leg_b (UAC)        │
  └────────────────┬────────────────────────┘
                   │
                   ▼
  ┌─────────────────────────────────────────┐
  │  sipstack / dialog / invite_dialog.rs   │
  │                                         │
  │  6. Leg B: Create + send SIP INVITE     │
  │  7. Start transaction state machine     │
  │  8. Create SDP offer (media-node IP)    │
  └────────────────┬────────────────────────┘
                   │
                   ▼
  ┌─────────────────────────────────────────┐
  │  sipstack / transport / udp.rs          │
  │                                         │
  │  9. Send SIP message via UDP/TCP/TLS    │
  └─────────────────────────────────────────┘
                   │
                   ▼
             SIP Phone / PSTN Gateway
```

---

## Implementation Phase

This module is developed alongside `sip-engine/b2bua` within **Phase 2** (Weeks 15-16):

| Week | Task |
|:---|:---|
| **15** | Basic REST router (originate, hangup, list), auth middleware |
| **16** | Transfer, hold/mute, WebSocket events, DTMF |
| **Phase 4** | Rate limiting, CORS, audit log, JWT support |
