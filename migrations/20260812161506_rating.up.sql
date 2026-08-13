-- Fixed taxonomy (seeded below with 'UP'), extensible by adding rows, not
-- migrations. No approved/deleted flag: not user-creatable, and ratings
-- capability explicitly excludes it from catalog-editing.
CREATE TABLE rating_type (
    id    BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name  TEXT NOT NULL
);

CREATE UNIQUE INDEX rating_type_name_key ON rating_type (name);

INSERT INTO rating_type (name) VALUES ('UP');

-- No approved/deleted flag: ratings are visible immediately (never
-- moderation-gated) and removal is a hard DELETE owned by the rating's
-- own creator (ratings capability) — see design.md's Decisions Log #3.
CREATE TABLE rating (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    store_product BIGINT NOT NULL REFERENCES store_product (id),
    rating_type   BIGINT NOT NULL REFERENCES rating_type (id),
    created_by    BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    created       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX rating_store_product_idx ON rating (store_product);
CREATE INDEX rating_created_by_idx ON rating (created_by);
-- A user's rating of a given type on a given store_product is a toggle,
-- not a stackable counter.
CREATE UNIQUE INDEX rating_unique_per_user_type ON rating (store_product, created_by, rating_type);
