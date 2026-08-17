#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["click>=8"]
# ///
"""Bootstrap the local (non-container) dev environment.

See bootstrap.md for details and prerequisites. The shebang above runs
this under `uv run --script`, which reads the inline `dependencies` block
and installs/caches `click` into an ephemeral venv on the fly — no
pre-installed `click` or manual venv needed, as long as `uv` itself is on
PATH (https://docs.astral.sh/uv/guides/scripts/). Run it either as
`./bootstrap.py app` (uv via the shebang) or `uv run bootstrap.py app`;
`python3 bootstrap.py app` also still works if `click` happens to already
be installed, just without uv managing that for you.

This is the "run cargo directly on the host" path. For a fully
containerized stack (app + db + caddy, no local Rust toolchain needed),
use `docker compose up --build` instead — see docker-compose.yml.

Subcommands:
    app     .env file + podman Postgres+PostGIS container + migrations
    stores  seed the db with OpenStreetMap shop=farm data (requires `app`
            already having been run — see scripts/seed_osm_farm_shops.py)
    all     app, then stores

Usage:
    ./bootstrap.py app
    ./bootstrap.py stores
    ./bootstrap.py all
"""

import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

import click

REPO_ROOT = Path(__file__).resolve().parent
ENV_FILE = REPO_ROOT / ".env"
ENV_EXAMPLE = REPO_ROOT / ".env.example"
SEED_SCRIPT = REPO_ROOT / "scripts" / "seed_osm_farm_shops.py"

CONTAINER_NAME = "bauernkarte_db"
DB_IMAGE = "docker.io/postgis/postgis:16-3.4"
DB_USER = "bauernkarte"
DB_PASSWORD = "dev"
DB_NAME = "bauernkarte"
# Host networking (rootless podman, no docker daemon in this env) means
# this port is the *host's* port, so it's set away from Postgres's
# default 5432 to avoid clashing with any other local Postgres.
DB_PORT = "5433"

CARGO_BIN = Path.home() / ".cargo" / "bin"


def run(cmd, **kwargs):
    click.echo(f"$ {' '.join(cmd)}")
    kwargs.setdefault("check", True)
    return subprocess.run(cmd, **kwargs)


def cargo_env():
    """cargo/rustc/sqlx aren't on the default PATH on this machine;
    prepend ~/.cargo/bin like the rest of local dev does."""
    env = os.environ.copy()
    if str(CARGO_BIN) not in env.get("PATH", ""):
        env["PATH"] = f"{CARGO_BIN}{os.pathsep}{env.get('PATH', '')}"
    return env


def require(cmd_name, path, hint):
    if shutil.which(cmd_name, path=path) is None:
        raise click.ClickException(f"`{cmd_name}` not found on PATH. {hint}")


def ensure_env_file():
    if ENV_FILE.exists():
        click.echo(f"{ENV_FILE.name} already exists, leaving it alone")
        return
    if not ENV_EXAMPLE.exists():
        raise click.ClickException(f"{ENV_EXAMPLE.name} is missing, can't seed .env")
    shutil.copyfile(ENV_EXAMPLE, ENV_FILE)
    # .env.example ships the production-shaped defaults (port 5432,
    # SECURE_COOKIES=true); local dev needs this container's port and
    # no TLS requirement on the session cookie (see README).
    text = ENV_FILE.read_text()
    text = text.replace(":5432/", f":{DB_PORT}/").replace(
        "SECURE_COOKIES=true", "SECURE_COOKIES=false"
    )
    ENV_FILE.write_text(text)
    click.echo(f"wrote {ENV_FILE.name} from {ENV_EXAMPLE.name} (port -> {DB_PORT}, "
               f"SECURE_COOKIES -> false)")


def container_exists():
    return subprocess.run(
        ["podman", "container", "exists", CONTAINER_NAME]
    ).returncode == 0


def container_running():
    result = subprocess.run(
        ["podman", "inspect", "-f", "{{.State.Running}}", CONTAINER_NAME],
        capture_output=True, text=True,
    )
    return result.returncode == 0 and result.stdout.strip() == "true"


def ensure_container():
    if container_exists():
        if container_running():
            click.echo(f"{CONTAINER_NAME} already running")
        else:
            click.echo(f"{CONTAINER_NAME} exists but is stopped, starting it")
            run(["podman", "start", CONTAINER_NAME])
        return
    click.echo(f"creating {CONTAINER_NAME} ({DB_IMAGE})")
    run([
        "podman", "run", "-d",
        "--name", CONTAINER_NAME,
        "--network", "host",
        "-e", f"POSTGRES_USER={DB_USER}",
        "-e", f"POSTGRES_PASSWORD={DB_PASSWORD}",
        "-e", f"POSTGRES_DB={DB_NAME}",
        "-e", f"PGPORT={DB_PORT}",
        DB_IMAGE,
    ])


def wait_for_db(timeout=60):
    click.echo("waiting for postgres to accept connections...")
    deadline = time.time() + timeout
    while time.time() < deadline:
        result = subprocess.run(
            ["podman", "exec", CONTAINER_NAME, "pg_isready",
             "-U", DB_USER, "-d", DB_NAME, "-p", DB_PORT],
            capture_output=True,
        )
        if result.returncode == 0:
            click.echo("postgres is ready")
            return
        time.sleep(1)
    raise click.ClickException(
        f"postgres did not become ready within {timeout}s "
        f"(check `podman logs {CONTAINER_NAME}`)"
    )


def database_url_from_env_file():
    for line in ENV_FILE.read_text().splitlines():
        if line.startswith("DATABASE_URL="):
            return line.split("=", 1)[1]
    raise click.ClickException(f"DATABASE_URL not set in {ENV_FILE.name}")


def run_migrations(env):
    env = {**env, "DATABASE_URL": database_url_from_env_file()}
    run(["sqlx", "migrate", "run"], cwd=REPO_ROOT, env=env)


def do_app():
    require("podman", None,
             "install podman, or adapt DB_IMAGE/run args in this script for docker.")
    ensure_env_file()
    ensure_container()
    wait_for_db()

    env = cargo_env()
    require("sqlx", env["PATH"],
             "install it with: cargo install sqlx-cli --no-default-features --features postgres")
    run_migrations(env)

    click.echo()
    click.echo("app bootstrap complete. Next:")
    click.echo(f"  export PATH=\"{CARGO_BIN}:$PATH\"   # if cargo isn't already on PATH")
    click.echo("  cargo run")


def do_stores():
    """Seed the db with OpenStreetMap shop=farm data (design.md's public
    reference data, see scripts/seed_osm_farm_shops.py's own docstring
    for the field mapping/provenance). Requires `app` to have already
    written `.env` and migrated whatever database its `DATABASE_URL`
    points at — this doesn't do that itself, so a fresh environment
    should use `all` instead."""
    require("podman", None, "install podman (see the `app` subcommand).")
    if not ENV_FILE.exists():
        raise click.ClickException(
            f"{ENV_FILE.name} is missing — run `bootstrap.py app` "
            "(or `bootstrap.py all`) first."
        )
    if not SEED_SCRIPT.exists():
        raise click.ClickException(f"{SEED_SCRIPT} is missing")

    # `--live`: the seed script's own default (fetch-vs-replay-stdin
    # picked from whether *its* stdin is a tty) only works run directly
    # in a terminal — this subprocess's stdin isn't a tty regardless of
    # how bootstrap.py itself was invoked, so `--live` forces the fetch
    # explicitly instead of relying on that.
    click.echo("fetching OSM shop=farm data from Overpass and generating "
               "seed SQL (this hits a public API and can take a minute or "
               "two)...")
    try:
        seed = subprocess.run(
            [sys.executable, str(SEED_SCRIPT), "--live"],
            cwd=REPO_ROOT, capture_output=True, text=True, check=True,
        )
    except subprocess.CalledProcessError as exc:
        raise click.ClickException(f"seed script failed:\n{exc.stderr}")
    if seed.stderr:
        click.echo(seed.stderr.strip())

    # Applied through DATABASE_URL (same source of truth as
    # `run_migrations`) rather than `podman exec` into CONTAINER_NAME, so
    # this seeds whichever database the app itself talks to — the
    # standalone container `app` creates, or the docker-compose `db`
    # service if that's what .env points at. `psql` isn't a host
    # dependency here: DB_IMAGE is already pulled and ships one, so a
    # throwaway `--network host` container acts as the client.
    database_url = database_url_from_env_file()
    click.echo(f"applying seed SQL against {database_url}...")
    run(
        ["podman", "run", "--rm", "-i", "--network", "host", DB_IMAGE,
         "psql", database_url, "-v", "ON_ERROR_STOP=1", "-q"],
        input=seed.stdout, text=True,
    )
    click.echo("stores seeded.")


@click.group()
def cli():
    """Bootstrap the local (non-container) BauernKarte dev environment."""


@cli.command()
def app():
    """podman Postgres+PostGIS container, .env, migrations."""
    do_app()


@cli.command()
def stores():
    """Seed the db with OSM shop=farm data (requires `app` already run)."""
    do_stores()


@cli.command(name="all")
def all_():
    """app, then stores."""
    do_app()
    do_stores()


if __name__ == "__main__":
    cli()
