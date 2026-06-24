ALTER TABLE oidc_auth_requests ADD COLUMN site_return_url TEXT;
ALTER TABLE registry_email_auth_requests ADD COLUMN site_return_url TEXT;

CREATE TABLE IF NOT EXISTS subdomains (
    id TEXT PRIMARY KEY,
    subdomain TEXT NOT NULL UNIQUE,
    asn TEXT NOT NULL,
    effective_mnt TEXT NOT NULL,
    nameservers TEXT NOT NULL,
    description TEXT,
    cf_record_ids TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_subdomains_asn ON subdomains (asn);
CREATE INDEX IF NOT EXISTS idx_subdomains_subdomain ON subdomains (subdomain);
