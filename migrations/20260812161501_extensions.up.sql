-- Enable PostGIS (store.position geography) and citext (case-insensitive
-- user.email) ahead of any table that depends on them.
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS citext;
