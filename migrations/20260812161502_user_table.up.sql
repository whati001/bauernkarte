-- `user` is a reserved-ish word in Postgres tooling but not a keyword, so
-- it's quoted consistently everywhere it's referenced (matches db_schema.txt
-- naming; design.md keeps the same entity name).
CREATE TABLE "user" (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name        TEXT NOT NULL,
    email       CITEXT NOT NULL,
    pwd_hash    TEXT NOT NULL, -- Argon2id encoded hash (~90-100 chars); see design.md §9.1
    verified    BOOLEAN NOT NULL DEFAULT false,
    created     TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX user_email_key ON "user" (email);
