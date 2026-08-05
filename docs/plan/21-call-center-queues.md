# 21 — Call Center Queues (ACD Service) Architecture

## 1. Executive Summary & Purpose

The **Call Center Queue Service** is a standalone binary (`bins/call-center/`) that handles Automatic Call Distribution (ACD), queuing incoming callers, playing hold music, distributing calls to agents based on routing strategies, and supporting queue callbacks.

---

## 2. Process Architecture & gRPC Pipeline

```text
Incoming Call ──► sip-engine ──gRPC: EnqueueCaller──► call-center (.exe)
                                                            │
                                                            ├── 1. Hold Music via media-node
                                                            ├── 2. Agent Strategy Evaluation
                                                            ├── 3. Ring Agent Endpoint(s)
                                                            └── 4. Bridge Caller & Agent
```

---

## 3. Core Features

### A. Distribution Strategies
- **Ring-All**: Rings all available agents simultaneously; first to answer gets the call.
- **Round-Robin**: Distributes calls sequentially across active agents.
- **Least-Recent**: Routes to the agent who has been idle the longest.
- **Random**: Randomly selects an available agent.

### B. Queue Callback Request
- Allows callers to press a digit (e.g., `1`) to hang up while retaining their position in queue.
- `call-center` automatically originates an outbound call to the customer when an agent becomes free.

### C. Real-Time Agent State Management
- Agent statuses: `Available`, `OnCall`, `WrapUp`, `Paused` (Break), `Offline`.
- Dynamic agent login/logout via SIP feature codes or REST API (`POST /api/v1/queues/{id}/agents/login`).

---

## 4. Implementation Phase

Scheduled for **Phase 4 (Weeks 31-32)** as an optional enterprise service binary.
