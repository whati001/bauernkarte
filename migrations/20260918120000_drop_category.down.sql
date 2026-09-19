-- Recreates the table and column, not the data: the rows are gone, so
-- every surviving product is re-pointed at one placeholder category.
CREATE TABLE category (
    id           BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name         TEXT NOT NULL,
    icon         TEXT,
    created_by   BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    modified_by  BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    created      TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX category_name_key ON category (name);

INSERT INTO category (name, icon) VALUES ('Sonstiges', '🧺');

ALTER TABLE product ADD COLUMN category BIGINT REFERENCES category (id);
UPDATE product SET category = (SELECT id FROM category LIMIT 1);
ALTER TABLE product ALTER COLUMN category SET NOT NULL;

CREATE INDEX product_category_idx ON product (category);
