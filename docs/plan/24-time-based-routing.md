# 24 — Time-Based Routing & Calendar Schedules

## 1. Executive Summary & Purpose

The Time-Based Routing module (`bins/sip-engine/src/routing/calendar.rs`) evaluates incoming call timestamps against defined business hours, weekend schedules, and public holiday calendars to determine dynamic call routing.

---

## 2. Rule Evaluation Engine

```text
Incoming Call ──► Time-Condition Check (Current Time vs. Rule Schedule)
                         │
                         ├── Match: Business Hours ──► Route to Ring Group / IVR
                         └── Match: After Hours / Holiday ──► Route to Voicemail / On-Call Mobile
```

---

## 3. Configuration & Schedule Definitions

Schedules are defined in SQLite and evaluated in-memory:

- **Weekly Schedules**: e.g., Monday–Friday, 09:00–18:00.
- **Holiday Override Schedules**: Fixed date ranges (e.g., Dec 25, Jan 1, national holidays) overriding standard weekly rules.
- **Manual Mode Overrides**: Forced Open / Forced Closed toggled instantly via REST API or SIP feature codes (`*70` / `*71`).

---

## 4. Implementation Phase

Scheduled for **Phase 3 (Weeks 19-20)** alongside IVR and Advanced Dialplan routing.
