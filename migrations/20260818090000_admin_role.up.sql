-- Adds the admin role and seeds the one bootstrap admin account.
--
-- The password is deliberately NOT set here. A migration is static SQL:
-- it can't read `.env` and it can't run Argon2, and a pre-computed hash
-- checked into the repository would be a published password. Instead the
-- row is created with an empty `pwd_hash`, which `PasswordHash::new`
-- fails to parse, so `verify_password` returns false and the account
-- cannot be logged into. On its *first* startup the server fills it in
-- from `ADMIN_PASSWORD` (see `src/auth/admin_seed.rs`) and never touches
-- it again — so a password changed later via /account survives restarts.
ALTER TABLE "user" ADD COLUMN admin BOOLEAN NOT NULL DEFAULT false;

-- Partial: admins are a handful of rows in a table of many, and the
-- moderation queue's "who can act on this" lookups only ever ask for the
-- true side.
CREATE INDEX user_admin_idx ON "user" (id) WHERE admin;

INSERT INTO "user" (name, email, pwd_hash, verified, admin)
VALUES ('BauernKarte Admin', 'bauernkarte@rehka.dev', '', true, true)
ON CONFLICT (email) DO UPDATE SET admin = true;
