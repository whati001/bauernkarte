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

-- Fixed taxonomy, seeded/managed by an admin directly in the DB — not
-- user-creatable in v1, so no approved/deleted flag (content-moderation
-- capability's scope explicitly excludes category).
CREATE TABLE category (
    id           BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name         TEXT NOT NULL,
    created_by   BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    modified_by  BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    created      TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX category_name_key ON category (name);

CREATE TABLE product (
    id           BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    category     BIGINT NOT NULL REFERENCES category (id),
    name         TEXT NOT NULL,
    description  TEXT,
    approved     BOOLEAN NOT NULL DEFAULT false,
    deleted      BOOLEAN NOT NULL DEFAULT false,
    created_by   BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    modified_by  BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    created      TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX product_category_idx ON product (category);
CREATE INDEX product_created_by_idx ON product (created_by);
CREATE INDEX product_modified_by_idx ON product (modified_by);
