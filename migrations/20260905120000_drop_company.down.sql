-- Recreates the table and column, not the data: the rows are gone, so
-- every surviving store is re-pointed at one placeholder company.
ALTER TABLE edit_log DROP CONSTRAINT edit_log_entity_type_check;
ALTER TABLE edit_log ADD CONSTRAINT edit_log_entity_type_check
    CHECK (entity_type IN ('company', 'store', 'product', 'store_product', 'image'));

CREATE TABLE company (
    id           BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name         TEXT NOT NULL,
    description  TEXT,
    homepage     TEXT,
    approved     BOOLEAN NOT NULL DEFAULT false,
    deleted      BOOLEAN NOT NULL DEFAULT false,
    created_by   BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    modified_by  BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    created      TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX company_created_by_idx ON company (created_by);
CREATE INDEX company_modified_by_idx ON company (modified_by);

INSERT INTO company (name, approved) VALUES ('Unbekannt', true);

ALTER TABLE store ADD COLUMN company BIGINT REFERENCES company (id);
UPDATE store SET company = (SELECT id FROM company LIMIT 1);
ALTER TABLE store ALTER COLUMN company SET NOT NULL;

CREATE INDEX store_company_idx ON store (company);
