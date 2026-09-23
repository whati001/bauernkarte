-- The redesigned store info panel shows things the schema had no place
-- for: where the shop is as an address (not just a pin), who runs it, a
-- phone number, and a star rating for the shop as a whole. All optional — every existing row (and every OSM-seeded one)
-- stays valid, and the panel simply leaves out what isn't known.

ALTER TABLE store
    ADD COLUMN address     TEXT,
    ADD COLUMN phone       TEXT,
    ADD COLUMN owner_name  TEXT,
    ADD COLUMN owner_since SMALLINT CHECK (owner_since BETWEEN 1800 AND 2100),
    ADD COLUMN owner_bio   TEXT;

-- A photo is either of the place or a portrait of the person running it;
-- the portrait is what the "shop owner" block shows as its avatar. Same
-- moderation as any other upload.
ALTER TABLE image
    ADD COLUMN kind TEXT NOT NULL DEFAULT 'photo' CHECK (kind IN ('photo', 'owner'));

-- One 1-5 star rating per user per shop. Separate from `rating`, which
-- stays what it was: a per-offer "UP" heart.
CREATE TABLE store_review (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    store       BIGINT NOT NULL REFERENCES store (id),
    stars       SMALLINT NOT NULL CHECK (stars BETWEEN 1 AND 5),
    created_by  BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    created     TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX store_review_store_idx ON store_review (store);
CREATE UNIQUE INDEX store_review_one_per_user ON store_review (store, created_by);
