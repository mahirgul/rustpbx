# 16 — IVR (Interactive Voice Response) & Voicemail Modules

## 1. IVR (Interactive Voice Response) Module Architecture

The IVR module handles incoming automated attendant menus (e.g., *"Press 1 for Sales, Press 2 for Support"*), audio prompts, and DTMF tone capturing.

```text
Incoming Call → sip-engine → IVR Session
                                │
                                ├── 1. Request prompt playback from media-node
                                ├── 2. Collect DTMF digits (RFC 4733 / In-band)
                                └── 3. Execute dialplan routing based on digit
```

### Module Location & Structure

- **Location**: `bins/sip-engine/src/b2bua/ivr/`
- **Key Components**:
  - `ivr_menu.rs`: Menu configuration model (prompts, timeout, invalid digit handling, max retries).
  - `dtmf_collector.rs`: Collects DTMF digits via SIP INFO or RFC 4733 RTP events from `sipstack`.
  - `prompt_player.rs`: Instructs `media-node` to play audio files (`.wav` / `.opus`) to the caller.

### Example IVR Configuration (TOML/SQLite)

```toml
[ivr.main_menu]
prompt = "/var/lib/rustpbx/prompts/welcome.wav"
timeout_secs = 5
max_failures = 3
invalid_prompt = "/var/lib/rustpbx/prompts/invalid.wav"

[ivr.main_menu.routes]
"1" = "extension:100"      # Sales Queue
"2" = "extension:200"      # Support Queue
"9" = "voicemail:general"  # Voicemail Box
"0" = "extension:0"        # Operator
```

---

## 2. Voicemail Service Architecture

The Voicemail service handles missed calls, audio recording, voicemail box management, email notifications, and MWI (Message Waiting Indicator) SIP NOTIFY events.

### Standalone Executable: `bins/voicemail/`

Voicemail is implemented as an independent binary service communicating with `sip-engine` via gRPC (`pbx-proto/voicemail.proto`):

```text
Missed Call → sip-engine ──gRPC: DepositVoicemail──► voicemail (.exe)
                                                          │
                                                          ├── 1. Play greeting prompt
                                                          ├── 2. Record audio message
                                                          ├── 3. Save to local disk / DB
                                                          ├── 4. Send MWI (RFC 3842) via sip-engine
                                                          └── 5. Send SMTP email with audio attachment
```

### Key Components of `voicemail` (.exe)

- **`box_manager.rs`**: Manages mailbox quotas, PIN verification, unread/read messages.
- **`mwi_notifier.rs`**: Triggers SIP NOTIFY (`Message-Waiting: yes`, `Voice-Messages: 1/0`) to update phone LED/icons via `sipstack` subscription layer.
- **`email_sender.rs`**: Asynchronous SMTP worker using `lettre` crate to send audio recording attachments to subscriber emails.
- **`stt_integration.rs`**: (Optional) Speech-to-Text integration for automated voicemail transcriptions in email notifications.

---

## 3. Storage Layout for Prompts & Voicemails

```text
/var/lib/rustpbx/
├── prompts/                   # IVR greetings & system prompts
│   ├── welcome.wav
│   ├── invalid_option.wav
│   └── leave_message.wav
│
└── voicemail/                 # Subscriber mailboxes
    └── boxes/
        └── 100/               # Extension 100 Mailbox
            ├── greeting.wav   # Personal greeting
            ├── msg_001.opus   # Unread voicemail message
            └── msg_002.opus
```

---

## 4. Implementation Phase

- **IVR Module (`sip-engine`)**: Phase 3 (Weeks 19-20) alongside Call Transfers & Advanced Dialplan.
- **Voicemail Executable (`voicemail`)**: Phase 3 (Weeks 21-22) alongside RFC 6665 Event Framework & MWI notifications.
