-- SQLite Database Schema for RustPBX Initial Setup

-- Extensions / Subscribers Table
CREATE TABLE IF NOT EXISTS extensions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    extension_number TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    display_name TEXT NOT NULL,
    email TEXT,
    record_calls INTEGER NOT NULL DEFAULT 0, -- 0 = false, 1 = true
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Trunks (PSTN / Outbound SIP Gateways)
CREATE TABLE IF NOT EXISTS trunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    trunk_name TEXT NOT NULL UNIQUE,
    sip_server TEXT NOT NULL,
    port INTEGER NOT NULL DEFAULT 5060,
    auth_username TEXT,
    auth_password TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Dialplan Rules Table
CREATE TABLE IF NOT EXISTS dialplan_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_name TEXT NOT NULL,
    pattern TEXT NOT NULL, -- e.g. "^1[0-9]{2}$" for extensions 100-199
    destination_type TEXT NOT NULL, -- "extension", "trunk", "ivr", "voicemail"
    destination_target TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 1,
    is_active INTEGER NOT NULL DEFAULT 1
);

-- Seed Sample Test Extensions (100 and 200)
INSERT OR IGNORE INTO extensions (extension_number, password, display_name, email, record_calls) 
VALUES ('100', '100100', 'Alice (Ext 100)', 'alice@pbx.local', 1);

INSERT OR IGNORE INTO extensions (extension_number, password, display_name, email, record_calls) 
VALUES ('200', '200200', 'Bob (Ext 200)', 'bob@pbx.local', 0);

-- Seed Default Dialplan Rule for Extensions (1xx, 2xx)
INSERT OR IGNORE INTO dialplan_rules (rule_name, pattern, destination_type, destination_target, priority)
VALUES ('Local Extensions', '^[1-2][0-9]{2}$', 'extension', 'self', 1);

-- Live Active SIP Registrations Table
CREATE TABLE IF NOT EXISTS sip_registrations (
    extension_number TEXT PRIMARY KEY,
    user_agent TEXT,
    contact_uri TEXT NOT NULL,
    source_ip TEXT NOT NULL,
    source_port INTEGER NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
