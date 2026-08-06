-- SQLite Comprehensive Production Schema for RustPBX

-- 1. Extensions / Subscribers Table
CREATE TABLE IF NOT EXISTS extensions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    extension_number TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    display_name TEXT NOT NULL,
    email TEXT,
    record_calls INTEGER NOT NULL DEFAULT 0, -- 0 = false, 1 = true
    is_active INTEGER NOT NULL DEFAULT 1,
    qualify_frequency INTEGER NOT NULL DEFAULT 60, -- SIP OPTIONS ping frequency in seconds (0 = disabled)
    nat_mode TEXT NOT NULL DEFAULT 'auto', -- 'auto', 'force_rport', 'stun', 'disabled'
    min_expires INTEGER NOT NULL DEFAULT 60,
    max_expires INTEGER NOT NULL DEFAULT 3600,
    auth_required INTEGER NOT NULL DEFAULT 1, -- Digest auth required for this extension
    max_concurrent_logins INTEGER NOT NULL DEFAULT 1,
    allowed_transport TEXT NOT NULL DEFAULT 'udp,tcp,tls,ws',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 2. Live Active SIP Registrations Table
CREATE TABLE IF NOT EXISTS sip_registrations (
    extension_number TEXT PRIMARY KEY,
    user_agent TEXT,
    contact_uri TEXT NOT NULL,
    source_ip TEXT NOT NULL,
    source_port INTEGER NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 3. Trunks (PSTN / Outbound SIP Gateways)
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

-- 4. Dialplan Rules Table
CREATE TABLE IF NOT EXISTS dialplan_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_name TEXT NOT NULL,
    pattern TEXT NOT NULL, -- e.g. "^1[0-9]{2}$" for extensions 100-199
    destination_type TEXT NOT NULL, -- "extension", "trunk", "ivr", "queue", "voicemail"
    destination_target TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 1,
    is_active INTEGER NOT NULL DEFAULT 1,
    UNIQUE(rule_name, pattern)
);

-- 5. IVR Menus Table
CREATE TABLE IF NOT EXISTS ivr_menus (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    announcement_file TEXT NOT NULL, -- e.g. "welcome.wav"
    timeout_seconds INTEGER NOT NULL DEFAULT 10,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 6. IVR Options (DTMF Key Mapping)
CREATE TABLE IF NOT EXISTS ivr_options (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ivr_id INTEGER NOT NULL,
    digit TEXT NOT NULL, -- "1", "2", "0", "*"
    action_type TEXT NOT NULL, -- "extension", "queue", "voicemail"
    action_target TEXT NOT NULL,
    FOREIGN KEY (ivr_id) REFERENCES ivr_menus(id) ON DELETE CASCADE
);

-- 7. Call Center Queues Table
CREATE TABLE IF NOT EXISTS call_queues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    queue_number TEXT NOT NULL UNIQUE,
    queue_name TEXT NOT NULL,
    strategy TEXT NOT NULL DEFAULT 'ring-all', -- 'ring-all', 'round-robin', 'least-recent'
    music_on_hold TEXT DEFAULT 'default.wav',
    max_wait_seconds INTEGER NOT NULL DEFAULT 300,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 8. Queue Agents Mapping
CREATE TABLE IF NOT EXISTS queue_agents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    queue_id INTEGER NOT NULL,
    extension_number TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (queue_id) REFERENCES call_queues(id) ON DELETE CASCADE
);

-- 9. Voicemail Boxes
CREATE TABLE IF NOT EXISTS voicemails (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    extension_number TEXT NOT NULL UNIQUE,
    pin_code TEXT NOT NULL DEFAULT '1234',
    email_notification INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 10. Call Audio Recordings Metadata Table
CREATE TABLE IF NOT EXISTS call_recordings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    call_id TEXT NOT NULL,
    caller TEXT NOT NULL,
    callee TEXT NOT NULL,
    format TEXT NOT NULL DEFAULT 'wav', -- 'wav', 'opus', 'mp3', 'gsm'
    file_path TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL DEFAULT 0,
    duration_seconds INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Seed Sample Test Extensions (100 and 200)
INSERT OR IGNORE INTO extensions (extension_number, password, display_name, email, record_calls) 
VALUES ('100', '100100', 'Alice (Ext 100)', 'alice@pbx.local', 1);

INSERT OR IGNORE INTO extensions (extension_number, password, display_name, email, record_calls) 
VALUES ('200', '200200', 'Bob (Ext 200)', 'bob@pbx.local', 0);

-- Seed Default Dialplan Rule for Extensions (1xx, 2xx)
INSERT OR IGNORE INTO dialplan_rules (rule_name, pattern, destination_type, destination_target, priority)
VALUES ('Local Extensions', '^[1-2][0-9]{2}$', 'extension', 'self', 1);
