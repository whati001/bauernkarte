CREATE TABLE store (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    company       BIGINT NOT NULL REFERENCES company (id),
    name          TEXT NOT NULL,
    position      GEOGRAPHY(Point, 4326) NOT NULL,
    openinghours  TEXT,
    approved      BOOLEAN NOT NULL DEFAULT false,
    deleted       BOOLEAN NOT NULL DEFAULT false,
    created_by    BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    modified_by   BIGINT REFERENCES "user" (id) ON DELETE SET NULL,
    created       TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX store_company_idx ON store (company);
CREATE INDEX store_created_by_idx ON store (created_by);
CREATE INDEX store_modified_by_idx ON store (modified_by);
-- Distance queries (ST_DWithin/ST_Distance) are the whole point of
-- store-search; this index is load-bearing for that capability.
CREATE INDEX store_position_gist_idx ON store USING GIST (position);
