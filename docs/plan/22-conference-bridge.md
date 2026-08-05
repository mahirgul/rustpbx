# 22 — Multi-Party Audio Conference Bridge Architecture

## 1. Executive Summary & Purpose

The **Conference Bridge** module operates within `media-node` (`media-node/src/conference/`) to provide high-capacity multi-party audio conferencing with real-time audio mixing.

---

## 2. Audio Mixing Pipeline

```text
Participant 1 Audio (RTP) ──┐
Participant 2 Audio (RTP) ──┼──► Audio Mixer ──► N-1 Summer Engine ──► Custom RTP Stream per Participant
Participant 3 Audio (RTP) ──┘    (PCM16)         (Subtracts own voice)
```

To prevent audio echo and feedback, the mixer uses an **N-1 summing algorithm**: for each participant $i$, the output audio stream equals the sum of all participant audio minus participant $i$'s own audio.

---

## 3. Core Features

- **PIN Authentication**: Optional moderator and attendee PINs.
- **Mute / Unmute Controls**: Moderator can mute individual participants via REST API or DTMF (`*5`).
- **Active Speaker Detection**: Highlights current active speakers in the WebSocket event stream.
- **Conference Recording**: Records the mixed multi-party audio stream directly to disk/ClickHouse.

---

## 4. Implementation Phase

Scheduled for **Phase 3 (Weeks 23-24)** alongside `media-node` audio enhancements.
