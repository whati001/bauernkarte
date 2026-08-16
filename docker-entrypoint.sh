#!/bin/sh
# Applies pending migrations against DATABASE_URL, then hands off to the
# app binary. Migrations are otherwise a manual `sqlx migrate run` step
# (see README) — running them here is what makes `docker compose up`
# produce a working app against a freshly created `db` service instead
# of crash-looping against an empty schema.
set -eu

./sqlx migrate run --source ./migrations
exec ./bauernkarte
