CREATE TABLE image (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    store_product BIGINT NOT NULL REFERENCES store_product (id),
    image         BYTEA NOT NULL,
    mime_type     TEXT NOT NULL,
    description   TEXT,
    approved      BOOLEAN NOT NULL DEFAULT false,
    deleted       BOOLEAN NOT NULL DEFAULT false,
    created_by    BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    modified_by   BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    created       TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX image_store_product_idx ON image (store_product);
CREATE INDEX image_created_by_idx ON image (created_by);
CREATE INDEX image_modified_by_idx ON image (modified_by);
