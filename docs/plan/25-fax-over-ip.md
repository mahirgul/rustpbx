# 25 — T.38 FAX over IP (FoIP) Gateway Architecture

## 1. Executive Summary & Purpose

The FAX Gateway module (`media-node/src/fax/`) provides legacy and digital FAX processing via T.38 Fax over IP protocol, supporting **Fax-to-Email** (converting incoming faxes to PDF) and **Email-to-Fax** (sending PDF files as outbound faxes).

---

## 2. Architecture & Workflows

### Fax-to-Email (Inbound)
```text
Incoming T.38 FAX ──► media-node / fax ──► TIFF / PDF Converter ──► SMTP Email with PDF Attachment
```

### Email-to-Fax (Outbound)
```text
Outbound Email (PDF) ──► web-admin / SMTP Receiver ──► PDF to T.38 ──► SIP / T.38 Trunk
```

---

## 3. Key Components

- **T.38 Protocol Engine**: Pure Rust implementation of ITU-T T.38 recommendation for real-time facsimile over IP.
- **T.30 Protocol State Machine**: Handles fax handshake, page negotiation, error correction mode (ECM).
- **Image Converter**: Decodes T.4/T.6 TIFF images and renders multi-page PDF documents.

---

## 4. Implementation Phase

Scheduled for **Phase 4 (Weeks 29-30)** as an optional legacy enterprise gateway feature.
