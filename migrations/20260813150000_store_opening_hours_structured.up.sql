-- Opening hours move from a free-text field to a structured weekly
-- schedule: a JSONB array of `{"day": <1-7>, "open": "HH:MM", "close": "HH:MM"}`
-- objects (ISO 8601 weekday numbering, 1 = Monday .. 7 = Sunday; a day
-- with no entry is closed) — see src/opening_hours.rs for the shape and
-- src/models.rs's `DayHours`. Existing free-text values (arbitrary
-- prose) can't be parsed into that structure automatically, so they're
-- dropped rather than guessed at — same tradeoff already made for the
-- `store_product.price` column removal.
ALTER TABLE store ALTER COLUMN openinghours TYPE JSONB USING NULL::jsonb;
