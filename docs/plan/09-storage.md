# 09 — Storage Architecture & Management API

## Storage Architecture

RustPBX uses a multi-tier storage strategy optimized for different data lifecycles:

| Data Domain | Technology | Storage Format / Access Pattern |
|:---|:---|:---|
| **Extensions, Trunks, Dialplan, Users** | **SQLite (WAL Mode)** | Relational tables. Loaded into `sip-engine` in-memory cache on startup with atomic reload triggers via Axum REST / gRPC. |
| **Call Detail Records (CDR) & Analytics** | **ClickHouse** | Columnar NoSQL data store. Ring buffer pipeline: `tokio::mpsc` inside `sip-engine` → batch insert into ClickHouse every 5s. High compression (>85%) and sub-second analytical queries across millions of CDR rows. |
| **Audio Call Recordings** | **Local Disk / Storage** | Configurable format (WAV, Opus, MP3, GSM). Stored on local filesystem in structured date folders (`/var/lib/rustpbx/recordings/YYYY/MM/DD/`). File path references stored in ClickHouse CDR records. |
| **Active Call State** | **In-Memory `DashMap`** | Process-local, ephemeral call bridges inside `sip-engine`. |
| **Crash Recovery Snapshots** | **SQLite (WAL Mode)** | Minimal dialog state written every 5s. Cleared automatically after graceful BYE termination. |

---

## Management & IPC

`sip-engine` exposes a unified networking stack on a single management port:

1. **gRPC (`Tonic`):** High-speed inter-process communication (`media-node` orchestration, `webrtc-gateway` coordination, call control).
2. **WebSocket Streaming:** Real-time call state events (`CallStarted`, `CallAnswered`, `CallEnded`, `DTMFReceived`).
3. **REST API (`Axum`):** External call control, health checks (`/health`), Prometheus metrics (`/metrics`).
