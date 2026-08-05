# 27 — Auto-Provisioning System Architecture for IP Phones

## 1. Executive Summary & Purpose

The **Auto-Provisioning System** (`bins/web-admin/src/provisioning/` or standalone module) automates mass deployment of desktop IP phones and ATA devices.

When an unconfigured IP phone is plugged into the network, it fetches its configuration file dynamically from RustPBX using its **MAC Address**, eliminating manual phone configuration.

---

## 2. Supported IP Phone Brands & Protocols

Supported major IP phone vendors and their config file formats:

| Vendor | Supported Protocols | Config File Naming Pattern | Format |
|:---|:---|:---|:---|
| **Yealink** | HTTP / HTTPS / TFTP | `{MAC}.cfg` or `y0000000000xx.cfg` | Key-Value (INI style) |
| **Grandstream** | HTTP / HTTPS / TFTP | `cfg{MAC}.xml` | XML / Encrypted Binary |
| **Fanvil** | HTTP / HTTPS / TFTP | `{MAC}.xml` | XML |
| **Snom** | HTTP / HTTPS | `snom{Model}-{MAC}.xml` | XML |
| **Cisco / Linksys** | HTTP / HTTPS / TFTP | `SPA{MAC}.xml` or `XMLDefault.cnf.xml` | XML |
| **Poly / Polycom** | HTTP / HTTPS / TFTP | `{MAC}.cfg` & `phone1.cfg` | XML |

---

## 3. Provisioning Workflows & Protocols

```text
                               ┌────────────────────────────────────────────────────────┐
                               │                     web-admin                          │
                               │           Auto-Provisioning Engine                     │
                               └──────────────────────────┬─────────────────────────────┘
                                                          │
     ┌────────────────────────────────────────────────────┼────────────────────────────────────────────────────┐
     ▼ (DHCP Option 66 / PNP / RPS)                       ▼ (HTTP/HTTPS Endpoint)                              ▼ (TFTP Server - Optional)
┌──────────┐                                      ┌──────────────┐                                     ┌──────────┐
│  DHCP    │                                      │ IP Phone     │                                     │ Legacy   │
│ Server   │                                      │ (Booting up) │                                     │ IP Phone │
└──────────┘                                      └──────┬───────┘                                     └──────────┘
                                                         │
                                                         ├── 1. Request config: GET /provisioning/00155d010203.cfg
                                                         ├── 2. Validate MAC address & Firmware version
                                                         ├── 3. Render template with Extension, SIP Password & Server IP
                                                         └── 4. Return HTTP 200 OK + Phone Config Text
```

### Discovery & Option Mechanisms

1. **DHCP Option 66 / 160**: DHCP server tells booting phones the provisioning URL (e.g., `http://pbx.company.com/provisioning/`).
2. **SIP PNP (Plug and Play / Multicast SUBSCRIBE)**: `sipstack` listens for multicast SUBSCRIBE requests (`Event: ua-profile`) from newly plugged local phones and sends NOTIFY with the provisioning URL.
3. **Vendor RPS (Redirection & Provisioning Service)**: Interoperability with Yealink RPS / Grandstream GDMS cloud servers pointing devices to RustPBX.

---

## 4. Security & Encryption

- **HTTP Basic / Digest Auth**: Secures configuration URLs so unauthorized devices cannot download SIP passwords.
- **Mutual TLS (mTLS)**: Validates client certificates embedded in IP phone hardware.
- **Config Sensitive Variable Masking**: Automatically encrypts or masks SIP passwords in provisioning logs.

---

## 5. Web Admin Interface (`web-admin`)

- **Device Directory**: Manage devices by MAC Address, Assigned Extension, Model, Firmware Version, and Status.
- **Template Editor**: Visual Jinja2/Handlebars-style template editor for custom phone key mappings (BLF keys, Speed dial, Softkeys).
- **Firmware Server**: Centralized firmware repository allowing one-click remote firmware upgrades for entire fleets of IP phones.
- **Remote Reboot**: Triggers SIP NOTIFY (`Event: check-sync` / `reboot`) via `sip-engine` to force phones to pull new configs immediately.

---

## 6. Implementation Phase Schedule

Scheduled for **Phase 4 (Week 31)** as part of the `web-admin` enterprise deployment suite.
