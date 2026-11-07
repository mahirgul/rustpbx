# 29 — Decoupling `web-admin` from the SIP Core

> **Status:** Planned (not implemented). Decided on 2026-11-07.
> **Goal:** `sip-engine` is a pure SIP server. It must run, be deployed, and be upgraded without `web-admin`, and `web-admin` must never touch SIP runtime state directly.

---

## 1. Current State (Assessment, 2026-11-07)

`web-admin` and `sip-engine` are already separate binaries, and `sip-engine` does not depend on the `web-admin` crate. The real coupling is the **shared SQLite file** `data/rustpbx.db`, which has become an implicit API:

| # | Coupling | Where |
|---|---|---|
| 1 | Both binaries run `data/schema.sql` via `include_str!` and duplicate the same `ALTER TABLE` migrations | `bins/web-admin/src/main.rs`, `bins/sip-engine/src/db/store.rs` |
| 2 | The `Extension` model is defined twice, independently | `bins/web-admin/src/api/extensions.rs`, `bins/sip-engine/src/db/store.rs` |
| 3 | `sip-engine` reloads **all** extensions from the DB on every REGISTER and INVITE (hot path depends on DB) | `sip_handler/register.rs`, `sip_handler/invite.rs` |
| 4 | Volatile registration state (`sip_registrations`) is persisted in SQLite and read directly by `web-admin` | `db/store.rs`, `web-admin/src/api/system.rs`, `extensions.rs` |
| 5 | The dashboard detects services by port probing and `pgrep`; `active_calls` is hard-coded to `0` | `web-admin/src/api/system.rs` |
| 6 | The `sip-engine` REST API (port 8085) has no authentication; `api.secret_key` is unused | `sip-engine/src/api/rest.rs` |

---

## 2. Target Architecture

```text
┌──────────────┐   gRPC (token auth)   ┌──────────────────────────────┐
│  web-admin   │ ────────────────────► │          sip-engine           │
│  (UI + REST) │ ◄──── event stream ── │  in-memory registrar/location │
└──────┬───────┘                       │  in-memory config cache       │
       │ owns config writes            └──────────────┬───────────────┘
       ▼                                              │ reads config via
┌──────────────┐                                      │ ConfigSource trait
│  pbx-store   │ ◄────────────────────────────────────┘
│ (schema,     │      SqliteSource | TomlFileSource | ApiSource
│  migrations) │
└──────────────┘
```

Rules:
- `sip-engine` never writes configuration and never reads `web-admin` tables on the SIP hot path.
- `web-admin` never reads SIP runtime state (registrations, calls) from the DB; it asks `sip-engine` over the control API.
- Exactly one component runs migrations.

---

## 3. Phases

### Phase D1 — `crates/pbx-store`
- Move schema, migrations (`sqlx::migrate!` with versioned files) and shared models (`Extension`, `Trunk`, `DialplanRule`) into one crate.
- Delete the duplicated `ALTER TABLE` blocks from both binaries.
- `web-admin` runs migrations; `sip-engine` opens the DB read-only.

### Phase D2 — In-memory state in `sip-engine`
- Registrar / location service backed by `DashMap` (see [30](30-sip-core-hardening-roadmap.md), M1). `sip_registrations` is no longer written per REGISTER (optional periodic snapshot for crash recovery, see [07](07-fault-recovery.md)).
- Config cache (extensions, trunks, dialplan) loaded at start-up and refreshed on `ReloadConfig` or by polling SQLite `PRAGMA data_version`.
- Credential lookup becomes an O(1) map access instead of a full table scan.

### Phase D3 — Authenticated control API
- New `crates/pbx-proto/proto/sip_admin.proto`:
  - `ListRegistrations`, `ListCalls`, `HangupCall`, `ReloadConfig`, `GetStats`
  - `WatchEvents` (server stream: registration and call state changes)
- Token authentication using `api.secret_key` (mTLS optional).
- `web-admin` dashboard uses this API; port probing, `pgrep` and direct `sip_registrations` reads are removed.

### Phase D4 — `ConfigSource` trait
- `trait ConfigSource { load_extensions, load_trunks, load_dialplan, watch }` with `SqliteSource` and `TomlFileSource` implementations (see [31](31-modular-plugin-and-codec-api.md)).
- `sip-engine` can run standalone from a TOML file, with no DB and no admin.
- `web-admin` may later move to a separate repository.

Feature flags are not required: binaries are already separate. Use workspace `default-members` and `cargo build -p sip-engine` for a lean SIP-only build.

---

## 4. Trade-offs

**Pros:** admin crashes or upgrades do not affect calls; smaller attack surface on the SIP host; explicit, versioned API contract; DB locks removed from the SIP hot path; enables multiple `sip-engine` nodes.

**Cons:** API versioning and config propagation (latency, consistency) must be designed; one more network hop for admin views; slightly less "copy one binary and run" simplicity.
