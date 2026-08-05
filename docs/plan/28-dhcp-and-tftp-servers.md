# 28 — Embedded DHCP & TFTP Server Modules for Auto-Provisioning

## 1. Executive Summary & Purpose

To enable true **Zero-Touch Provisioning** on isolated VoIP LANs/VLANs, RustPBX includes lightweight, high-performance embedded **DHCP** and **TFTP** server modules (`bins/web-admin/src/network/` or optional standalone binary).

This eliminates the need to configure external Windows Server / Linux DHCP or TFTP daemons for IP phone deployments.

---

## 2. Embedded DHCP Server Module (`dhcp.rs`)

A Pure Rust async UDP DHCP server implementing RFC 2131 & RFC 2132.

```text
IP Phone (Unconfigured) ──► DHCP DISCOVER (Broadcast)
                                     │
RustPBX DHCP Server     ◄── DHCP OFFER ─────────────────────────────┐
                                                                    │
                                    ├── IP Address Lease            │
                                    ├── Subnet Mask & Gateway       │
                                    ├── Option 66 (TFTP Server IP) ─┘
                                    └── Option 160 (HTTP Provisioning URL)
```

### Key DHCP Options Supported
- **Option 66 (`tftp-server-name`)**: Provides IP address or hostname of the TFTP/HTTP provisioning server (e.g., `192.168.1.10`).
- **Option 160 / 150**: Vendor-specific HTTP provisioning URL (`http://192.168.1.10:8080/provisioning/`).
- **Option 42 (`ntp-servers`)**: Synchronizes time on desktop IP phones.

---

## 3. Embedded TFTP Server Module (`tftp.rs`)

A Pure Rust async UDP TFTP server implementing RFC 1350, RFC 2347, RFC 2348 (blocksize option), and RFC 2349 (timeout option).

### Purpose
Legacy IP phones (e.g., Cisco SPA, old Polycom, old Snom devices) request configuration files and firmware binaries exclusively via TFTP (UDP Port 69).

```text
Legacy IP Phone ──► TFTP RRQ (Read Request): cfg00155d010203.xml ──► RustPBX TFTP Server
                                                                             │
Legacy IP Phone ◄── TFTP DATA (512 / 1428 Byte Blocks) ◄─────────────────────┘
```

### Key TFTP Engine Features
- **Asynchronous UDP Socket (Tokio)**: High-concurrency TFTP transfer handling multiple booting phones simultaneously.
- **Dynamic File Generator**: Directly maps incoming TFTP file requests (`cfg{MAC}.xml`) into the `web-admin` Auto-Provisioning template renderer without requiring static disk files.
- **Blocksize Extension (RFC 2348)**: Speeds up firmware transfers by expanding block size beyond 512 bytes.

---

## 4. Web Admin Configuration (`web-admin`)

Both services are fully managed via the Unified Web Admin Interface:

- **DHCP Subnet Pool Settings**: Enable/disable DHCP server per network interface, define IP range (`192.168.1.100` - `192.168.1.200`), lease duration, and Option 66 URL.
- **Static DHCP Leases**: Reserve static IP addresses by MAC Address.
- **TFTP Root & Logging**: Enable/disable TFTP server on Port 69, view live TFTP download activity logs.

---

## 5. Implementation Phase Schedule

Scheduled for **Phase 4 (Week 31)** alongside Auto-Provisioning & `web-admin` deployment tools.
