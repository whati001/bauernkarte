-- One fully filled-in shop for trying out the store info panel: owner,
-- address, phone, opening hours, seasonal offers, and a star rating.
-- Everything the OSM seed can't supply. Fictional data.
--
--   psql "$DATABASE_URL" -f scripts/demo_store.sql
--
-- Idempotent: does nothing if a store with this name already exists.

BEGIN;

INSERT INTO product (name, icon, approved) VALUES
    ('Boskoop-Äpfel', '🍎', true),
    ('Mirabellen', '🫐', true),
    ('Apfelcider', '🍺', true),
    ('Marillenmarmelade', '🫙', true)
ON CONFLICT (name) WHERE NOT deleted DO NOTHING;

DO $$
DECLARE
    sid BIGINT;
BEGIN
    IF EXISTS (SELECT 1 FROM store WHERE name = 'Obstgarten Maier') THEN
        RETURN;
    END IF;

    INSERT INTO store (name, position, kind, openinghours, approved,
                       address, phone, owner_name, owner_since, owner_bio)
    VALUES (
        'Obstgarten Maier',
        ST_SetSRID(ST_MakePoint(15.7086, 47.1043), 4326)::geography,
        'shop',
        '[{"day":5,"open":"09:00","close":"17:00"},
          {"day":6,"open":"09:00","close":"17:00"},
          {"day":7,"open":"09:00","close":"17:00"}]'::jsonb,
        true,
        'Apfelweg 3, 8200 Gleisdorf',
        '+43 3112 234 5678',
        'Hans-Jörg Maier',
        2005,
        'Hans bewirtschaftet 14 Hektar Obstgarten mit 60 Apfelsorten, dazu Steinobst und Beeren. Cider und Marmeladen entstehen direkt am Hof.'
    )
    RETURNING id INTO sid;

    INSERT INTO store_product (store, product, approved, seasonal_months)
    SELECT sid, p.id, true, v.months::jsonb
    FROM (VALUES
        ('Boskoop-Äpfel',     '[8,9,10,11,12]'),
        ('Mirabellen',        '[7,8,9]'),
        ('Apfelcider',        NULL),
        ('Marillenmarmelade', NULL)
    ) AS v(name, months)
    JOIN product p ON p.name = v.name AND NOT p.deleted;

    -- 138 anonymous ratings averaging 4.7 (104 x 5, 26 x 4, 8 x 3).
    INSERT INTO store_review (store, stars)
    SELECT sid, s FROM (
        SELECT 5 AS s FROM generate_series(1, 104)
        UNION ALL SELECT 4 FROM generate_series(1, 26)
        UNION ALL SELECT 3 FROM generate_series(1, 8)
    ) AS r;
END $$;

COMMIT;
