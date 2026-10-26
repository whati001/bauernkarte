-- Controller contact details shown only on the Datenschutz page.
-- This singleton is intentionally independent from the Impressum's site_info.
CREATE TABLE privacy_info (
    id                BIGINT PRIMARY KEY CHECK (id = 1),
    controller_name   TEXT NOT NULL DEFAULT '',
    street            TEXT NOT NULL DEFAULT '',
    postal_code       TEXT NOT NULL DEFAULT '',
    city              TEXT NOT NULL DEFAULT '',
    country           TEXT NOT NULL DEFAULT '',
    email             TEXT NOT NULL DEFAULT '',
    phone             TEXT NOT NULL DEFAULT '',
    modified_by       BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    modified          TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO privacy_info (id) VALUES (1);
