# 02 — `sipcore` Design — Pure SIP Parser & Types

## Purpose

Zero-I/O, WASM-compatible SIP message parsing and generation library.

**Dependencies:** `nom`, `bytes`, `sipcore-derives` only. **No** `tokio`, **no** networking.

**Why separate:** WASM Tier 1 plugins inside `sip-engine` need to parse and manipulate SIP messages. WASM cannot use `tokio` or system sockets. By keeping `sipcore` pure, it compiles to the `wasm32-wasi` target without modification.

---

## Memory Model: `Bytes` + Offset Indexing

```rust
use bytes::Bytes;

/// A parsed SIP message retains a reference-counted handle to the original buffer.
/// Header values are represented as Bytes slices into this buffer — true zero-copy.
pub struct SipMessage {
    raw: Bytes,                        // Ref-counted original buffer
    start_line: Range<usize>,          // Byte range into raw
    headers: Headers,                  // Indexed header collection
    body: Bytes,                       // Body (may be SDP, may be empty)
}

/// Headers: Vec for preservation of order + HashMap for O(1) lookup
pub struct Headers {
    entries: Vec<Header>,
    index: HashMap<HeaderName, SmallVec<[usize; 2]>>,
}

/// Each header stores byte ranges into the parent message buffer
pub struct Header {
    name: HeaderName,                  // Enum variant or Other(Bytes)
    raw_value: Bytes,                  // Slice of original buffer — zero-copy
}
```

**Key traits:**
- `Header::typed::<T: TypedHeader>() -> Result<T>` — On-demand deep parse (via nom)
- `TypedHeader::to_untyped() -> Header` — Serialize back to wire format
- `SipMessage::to_bytes() -> Bytes` — Serialize full message

---

## Parser Architecture (3-Stage)

```text
Network Buffer (Bytes)
       │
       ▼
┌──────────────────────────────┐
│  Stage 1: nom Tokenizer      │  Zero-copy byte slicing
│  Input: &[u8]                │  Output: Tokenizer<'a> with &'a [u8] refs
│  - No allocation             │  Header line folding support (RFC 3261 §7.3.1)
│  - No UTF-8 validation       │
└──────────────────────────────┘
       │
       ▼
┌──────────────────────────────┐
│  Stage 2: Bytes Mapping      │  Map &'a [u8] offsets → Bytes::slice()
│  Output: SipMessage          │  Headers stored as raw Bytes (untyped)
│  - Reference-counted buffer  │  O(1) header index constructed
│  - Zero-copy via Bytes       │
└──────────────────────────────┘
       │
       ▼ (on-demand, per header)
┌──────────────────────────────┐
│  Stage 3: .typed::<T>()      │  Deep nom parse of header value
│  Output: TypedFrom, TypedVia │  Only when application logic requires it
│  TypedContact, etc.          │
└──────────────────────────────┘
```
