# 05 — Distributed Architecture & Inter-Process Communication (IPC)

## Process Topology & IPC Map

```text
                                PUBLIC INTERNET
                                       |
                   +-------------------+-------------------+
                   |                                       |
        WSS / DTLS-SRTP (WebRTC)                     SIP (UDP/TCP/TLS)
                   |                                       |
                   v                                       v
     +--------------------------+                   +--------------------------+
     |   webrtc-gateway         |                   |   sip-engine             |
     |   (Edge Gateway)         |====[gRPC IPC]====>|   (Core B2BUA Engine)    |
     +--------------------------+                   +--------------------------+
                   |                                       |           |
                   |                                       |           | (In-Process <1ms)
                   |                              [gRPC]   |           v
                   |                                 |     |   +-------------------+
                   |                                 v     |   | WASM Plugins      |
                   |                    +-----------+------+   | (Tier 1 Fast-Path)|
                   |                    |  media-node         | | uses sipcore only |
                   +===[RTP/SRTP]======>|  (RTP Relay)        | +-------------------+
                                        +---------------------+
                                                                       |
                                                              [gRPC Async Events]
                                                                       |
       +-----------------------+-----------------------+---------------+
       |                       |                       |
       v                       v                       v
+--------------+       +---------------+       +---------------+
|web-admin     |       |call-center    |       |voicemail      |
| (Management) |       | (ACD / Queue) |       | (Audio/STT)   |
+--------------+       +---------------+       +---------------+
```

---

## IPC Protocol Matrix

| From → To | Protocol | Why |
|:---|:---|:---|
| `webrtc-gateway` → `sip-engine` | **gRPC** (`gateway_media.proto`) | Structured signaling: "ICE complete, SRTP keys=X, send SDP offer" |
| `sip-engine` → `media-node` | **gRPC** (`media_control.proto`) | "Open RTP port for Call X, apply SRTP decrypt key Y" |
| `webrtc-gateway` → `media-node` | **Direct RTP/SRTP** (UDP, data plane) | After sip-engine coordinates, raw media flows directly |
| `sip-engine` → service binaries | **gRPC events** (`call_events.proto`) | CallStarted, CallAnswered, CallEnded, QueueJoined |
| `web-admin` → `sip-engine` | **gRPC commands** | Hangup, Transfer, Config reload |

---

## WebRTC → SIP Media Path Coordination

```text
Step-by-step media path establishment for a WebRTC → SIP call:

  Browser                webrtc-gateway         sip-engine           media-node          SIP Phone
     │                        │                      │                    │                   │
     │── WSS SDP Offer ──────>│                      │                    │                   │
     │                        │── gRPC: NewCall ─────>│                    │                   │
     │                        │   (WebRTC SDP,        │                    │                   │
     │                        │    ICE candidates)    │                    │                   │
     │                        │                      │── SIP INVITE ──────────────────────────>│
     │                        │                      │   (rewritten SDP:                       │
     │                        │                      │    media IP = media-node)               │
     │                        │                      │<── 200 OK (SDP Answer) ────────────────│
     │                        │                      │                    │                   │
     │                        │                      │── gRPC: AllocateMedia ─>│               │
     │                        │                      │   (Call-ID, Leg-A SRTP  │               │
     │                        │                      │    keys, Leg-B RTP addr)│               │
     │                        │                      │                    │               │
     │                        │                      │── gRPC: MediaReady ─>│               │
     │                        │<── gRPC: ConnectMedia │   (media-node RTP     │               │
     │                        │   (media-node addr,  │    port for WebRTC     │               │
     │                        │    SRTP keys)        │    side)               │               │
     │                        │                      │                    │               │
     │<── WSS SDP Answer ─────│                      │                    │                   │
     │                        │                      │                    │                   │
     │══ DTLS-SRTP ══════════>│══ RTP (decrypted) ══════════════════════>│══ RTP ═══════════>│
     │   (encrypted)          │                      │                    │   (plain/SRTP)    │
```
