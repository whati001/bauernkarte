-- Contact/operator details shown on the Impressum page, editable by an
-- admin instead of being compiled into a template. An Impressum has to
-- be correct and current for whoever actually runs the site; baking it
-- into the repository means a redeploy every time an address changes,
-- and means a fork ships someone else's legal details.
--
-- One row, enforced by the CHECK rather than by convention: every query
-- reads `id = 1`, and a second row would silently become unreachable
-- data that looks editable in the admin form.
CREATE TABLE site_info (
    id                BIGINT PRIMARY KEY CHECK (id = 1),
    operator_name     TEXT NOT NULL DEFAULT '',
    street            TEXT NOT NULL DEFAULT '',
    postal_code       TEXT NOT NULL DEFAULT '',
    city              TEXT NOT NULL DEFAULT '',
    country           TEXT NOT NULL DEFAULT '',
    email             TEXT NOT NULL DEFAULT '',
    phone             TEXT NOT NULL DEFAULT '',
    -- Austrian specifics (§5 ECG, §25 MedienG). Optional: a hobby site
    -- has no UID and no Firmenbuch entry, and an empty field is simply
    -- left off the rendered page rather than shown blank.
    vat_id            TEXT NOT NULL DEFAULT '',
    register_number   TEXT NOT NULL DEFAULT '',
    responsible       TEXT NOT NULL DEFAULT '',
    purpose           TEXT NOT NULL DEFAULT '',
    modified_by       BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    modified          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Seeded empty, not with placeholder details. Invented legal contact
-- data that looks filled in is worse than an obviously blank page: the
-- Impressum handler renders a "not configured yet" notice while
-- `operator_name` is empty, which is visible and fixable.
INSERT INTO site_info (id) VALUES (1);
