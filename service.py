#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["click>=8"]
# ///
"""Bootstrap and deploy the containerized BauernKarte stack.

See service.md for details and prerequisites. The shebang above runs
this under `uv run --script`, which installs `click` into an ephemeral
venv on the fly — no manual venv needed, as long as `uv` is on PATH.
`python3 service.py ...` also works if `click` is already installed.

Everything is driven by `.env`: `env` creates it once (with random
secrets) and nothing ever rewrites it afterwards. docker-compose.yml
interpolates the same keys, and every other subcommand only reads it.
Change a value there and every subcommand follows.

Subcommands:
    env      create `.env` with random secrets (never overwrites it)
    db       start the compose `db` service and wait for it
    up       build and start the whole stack, wait until the app is healthy
    down     stop and remove the containers (data and image are kept)
    prepare  refresh the `.sqlx/` offline query cache (developers only)
    stores   seed OSM farm-shop/vending-machine data into the running db
    backup   dump the database to a file
    cleanup  tear the stack down: containers, images, volumes

Usage:
    ./service.py env
    ./service.py up
    ./service.py stores
    ./service.py down
"""

import os
import secrets
import shutil
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path
from urllib.parse import quote

import click

REPO_ROOT = Path(__file__).resolve().parent
ENV_FILE = REPO_ROOT / ".env"
SEED_SCRIPT = REPO_ROOT / "scripts" / "seed_osm_farm_shops.py"
DEMO_STORE = REPO_ROOT / "scripts" / "demo_store.sql"

CARGO_BIN = Path.home() / ".cargo" / "bin"

# Service names in docker-compose.yml, and the port Postgres listens on
# *inside* the compose network (DB_PORT is the host-side publish).
DB_SERVICE = "db"
DB_SERVICE_PORT = "5432"
APP_SERVICE = "app"

# Keys every `.env` must have. The DB_* values are the single source of
# truth for credentials — DATABASE_URL is derived from them, and
# docker-compose.yml interpolates them directly.
REQUIRED = [
    "DB_USER", "DB_PWD", "DB_NAME", "DB_HOST", "DB_PORT", "ADMIN_PASSWORD",
    "APP_PORT", "SECURE_COOKIES",
]

ENV_TEMPLATE = """\
# Generated once by `./service.py env` — safe to edit; nothing rewrites it. Keep it out of
# version control: it holds the database and admin passwords.
#
# Read by docker-compose.yml (compose auto-loads this file for `${{VAR}}`
# interpolation), service.py, and a host-side `dx serve`.

# --- database ----------------------------------------------------------
# docker-compose.yml passes these to the `db` service. Postgres only reads
# them when it first creates the `pgdata` volume — see "Resetting" in
# service.md before changing them on an existing install.
DB_USER={DB_USER}
DB_PWD={DB_PWD}
DB_NAME={DB_NAME}
# How the *host* reaches Postgres: compose publishes it on
# 127.0.0.1:DB_PORT. Containers use `db:{DB_SERVICE_PORT}` instead.
DB_HOST={DB_HOST}
DB_PORT={DB_PORT}

# Derived from the DB_* values above, for host-side `dx serve` and
# `cargo sqlx prepare`. Update it by hand after changing any DB_* value.
DATABASE_URL={DATABASE_URL}

# --- admin -------------------------------------------------------------
# Password for the seeded admin account (bauernkarte@rehka.dev). This is
# the only place it lives: the app applies it on every startup, and it
# can't be changed through the app. Edit it here and restart to change it.
ADMIN_PASSWORD={ADMIN_PASSWORD}

# --- app ---------------------------------------------------------------
# Host port compose publishes the `app` container on (8080 inside).
APP_PORT={APP_PORT}
# The app is served over plain HTTP, so this stays `false` — a `Secure`
# session cookie over HTTP makes login silently fail.
SECURE_COOKIES={SECURE_COOKIES}
"""


def run(cmd, **kwargs):
    click.echo(f"$ {' '.join(cmd)}")
    try:
        return subprocess.run(cmd, check=True, **kwargs)
    except subprocess.CalledProcessError as exc:
        raise click.ClickException(f"`{cmd[0]} ...` failed with exit code {exc.returncode}")


def require(cmd_name, path, hint):
    if shutil.which(cmd_name, path=path) is None:
        raise click.ClickException(f"`{cmd_name}` not found on PATH. {hint}")


def compose_cmd():
    """The compose implementation to drive: docker's v2 subcommand,
    podman's, or a standalone podman-compose/docker-compose binary."""
    for exe in ("docker", "podman"):
        if shutil.which(exe):
            probe = subprocess.run([exe, "compose", "version"], capture_output=True)
            if probe.returncode == 0:
                return [exe, "compose"]
    for exe in ("podman-compose", "docker-compose"):
        if shutil.which(exe):
            return [exe]
    raise click.ClickException(
        "no compose implementation found — install docker with the compose "
        "plugin, or podman with podman-compose."
    )


def database_url(cfg, host, port):
    """A connection string from `.env`'s credentials for a given
    host/port — the host-published one for tooling on the host,
    `db:5432` for anything inside the compose network."""
    user = quote(cfg["DB_USER"], safe="")
    pwd = quote(cfg["DB_PWD"], safe="")
    return f"postgres://{user}:{pwd}@{host}:{port}/{cfg['DB_NAME']}"


def render_env(values):
    values = dict(values)
    values["DB_SERVICE_PORT"] = DB_SERVICE_PORT
    values["DATABASE_URL"] = database_url(values, values["DB_HOST"], values["DB_PORT"])
    return ENV_TEMPLATE.format(**values)


def load_env():
    """Parse `.env` into a dict, and re-derive DATABASE_URL from the
    DB_* keys so they stay the single source of truth."""
    if not ENV_FILE.exists():
        raise click.ClickException(
            f"{ENV_FILE.name} is missing — run `./service.py env` first."
        )
    cfg = {}
    for line in ENV_FILE.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        cfg[key.strip()] = value.strip().strip('"').strip("'")

    missing = [k for k in REQUIRED if k not in cfg]
    if missing:
        raise click.ClickException(
            f"{ENV_FILE.name} is missing {', '.join(missing)} — it predates "
            "the current layout. Add them by hand (see .env.example)."
        )

    derived = database_url(cfg, cfg["DB_HOST"], cfg["DB_PORT"])
    if cfg.get("DATABASE_URL") != derived:
        click.echo(
            f"warning: {ENV_FILE.name}'s DATABASE_URL doesn't match its DB_* "
            f"values. Using {derived}; fix the file so a host-side `dx serve` "
            "reads the same one.",
            err=True,
        )
    cfg["DATABASE_URL"] = derived
    return cfg


def compose_env(cfg):
    """Environment for compose calls: real env vars take precedence over
    `.env` during interpolation, so passing the parsed values through
    makes this work the same on implementations that don't read `.env`
    themselves.

    Builds go to the Docker engine's own builder for the current context
    rather than whichever buildx builder happens to be selected — a
    `docker-container` builder runs in its own container with its own
    DNS, and can fail to reach the registry where the engine doesn't.
    Setting BUILDX_BUILDER yourself overrides this; podman ignores it."""
    env = {**os.environ, **cfg}
    if "BUILDX_BUILDER" not in env and shutil.which("docker"):
        context = subprocess.run(["docker", "context", "show"], capture_output=True, text=True)
        if context.returncode == 0 and context.stdout.strip():
            env["BUILDX_BUILDER"] = context.stdout.strip()
    return env


def db_is_ready(compose, cfg):
    return subprocess.run(
        compose + ["exec", "-T", DB_SERVICE, "pg_isready",
                   "-U", cfg["DB_USER"], "-d", cfg["DB_NAME"]],
        capture_output=True, env=compose_env(cfg),
    ).returncode == 0


def wait_for(what, check, logs_hint, timeout):
    click.echo(f"waiting for {what}...")
    deadline = time.time() + timeout
    while time.time() < deadline:
        if check():
            click.echo(f"{what} is ready")
            return
        time.sleep(2)
    raise click.ClickException(f"{what} did not become ready within {timeout}s ({logs_hint})")


def app_is_healthy(compose, cfg):
    probe = subprocess.run(
        compose + ["ps", "--format", "{{.Health}}", APP_SERVICE],
        capture_output=True, text=True, env=compose_env(cfg),
    )
    return probe.stdout.strip() == "healthy"


def require_running_db(compose, cfg):
    if not db_is_ready(compose, cfg):
        raise click.ClickException(
            f"the `{DB_SERVICE}` service isn't up and accepting connections "
            "— run `./service.py db` (or `up`) first."
        )


def psql_in_db(compose, cfg, sql):
    """Apply SQL with the `psql` inside the running `db` container — no
    host install needed, and no password: the image trusts connections
    over its local socket."""
    run(
        compose + ["exec", "-T", DB_SERVICE, "psql",
                   "-U", cfg["DB_USER"], "-d", cfg["DB_NAME"],
                   "-v", "ON_ERROR_STOP=1", "-q"],
        input=sql, text=True, env=compose_env(cfg),
    )


def do_env():
    # `.env` is the source of truth once it exists — edit it by hand.
    if ENV_FILE.exists():
        click.echo(f"{ENV_FILE.name} already exists, leaving it alone")
        return
    values = {
        "DB_USER": "bauernkarte",
        # URL-safe, so DATABASE_URL needs no escaping when read by hand.
        "DB_PWD": secrets.token_urlsafe(24),
        "DB_NAME": "bauernkarte",
        "DB_HOST": "127.0.0.1",
        "DB_PORT": "5434",
        "ADMIN_PASSWORD": secrets.token_urlsafe(18),
        "APP_PORT": "3000",
        "SECURE_COOKIES": "false",
    }
    ENV_FILE.write_text(render_env(values))
    ENV_FILE.chmod(0o600)
    click.echo(f"wrote {ENV_FILE.name} with generated secrets")
    click.echo(f"admin login: bauernkarte@rehka.dev / {values['ADMIN_PASSWORD']}")


def do_db():
    cfg = load_env()
    compose = compose_cmd()
    run(compose + ["up", "-d", DB_SERVICE], env=compose_env(cfg))
    wait_for("postgres", lambda: db_is_ready(compose, cfg),
             f"check `{' '.join(compose)} logs {DB_SERVICE}`", timeout=90)


def do_up():
    cfg = load_env()
    compose = compose_cmd()
    # The first build compiles the whole app (server + WASM client) and
    # takes a while; later ones reuse the build cache.
    run(compose + ["up", "-d", "--build"], env=compose_env(cfg))
    # The app runs pending migrations on startup, then reports healthy.
    wait_for("the app", lambda: app_is_healthy(compose, cfg),
             f"check `{' '.join(compose)} logs {APP_SERVICE}`", timeout=180)
    click.echo()
    click.echo(f"app is up on http://127.0.0.1:{cfg['APP_PORT']}")


def do_down():
    """Stop and remove the stack's containers. The `pgdata` volume and
    the built image stay, so `up` brings everything back as it was."""
    cfg = load_env()
    compose = compose_cmd()
    run(compose + ["down"], env=compose_env(cfg))
    click.echo("stack stopped — `./service.py up` starts it again.")


def do_prepare():
    cfg = load_env()
    compose = compose_cmd()
    require_running_db(compose, cfg)
    env = {**os.environ, "DATABASE_URL": cfg["DATABASE_URL"]}
    if str(CARGO_BIN) not in env.get("PATH", ""):
        env["PATH"] = f"{CARGO_BIN}{os.pathsep}{env.get('PATH', '')}"
    require("cargo", env["PATH"], "install Rust: https://rustup.rs")
    require("cargo-sqlx", env["PATH"],
            "install it with: cargo install sqlx-cli --no-default-features "
            "--features postgres,rustls")
    # The queries live behind the `server` feature, so prepare has to
    # build with it. It needs a migrated schema: the app migrates on
    # startup, `sqlx migrate run` does the same by hand.
    run(["cargo", "sqlx", "migrate", "run"], cwd=REPO_ROOT, env=env)
    run(["cargo", "sqlx", "prepare", "--", "--no-default-features", "--features", "server"],
        cwd=REPO_ROOT, env=env)
    click.echo(".sqlx/ refreshed — commit it with the queries that changed.")


def do_stores(demo):
    """Seed the db with OpenStreetMap farm-produce data — `shop=farm`
    plus farm-produce `amenity=vending_machine`s (see
    scripts/seed_osm_farm_shops.py for the field mapping/provenance)."""
    cfg = load_env()
    compose = compose_cmd()
    require_running_db(compose, cfg)

    # `--live`: the seed script's own default (fetch-vs-replay-stdin
    # picked from whether *its* stdin is a tty) doesn't work from a
    # subprocess, whose stdin is never a tty.
    click.echo("fetching OSM farm-shop and vending-machine data from Overpass "
               "and generating seed SQL (this hits a public API and can take "
               "a minute or two)...")
    try:
        seed = subprocess.run(
            [sys.executable, str(SEED_SCRIPT), "--live"],
            cwd=REPO_ROOT, capture_output=True, text=True, check=True,
        )
    except subprocess.CalledProcessError as exc:
        raise click.ClickException(f"seed script failed:\n{exc.stderr}")
    if seed.stderr:
        click.echo(seed.stderr.strip())
    psql_in_db(compose, cfg, seed.stdout)
    if demo:
        psql_in_db(compose, cfg, DEMO_STORE.read_text())
    click.echo("stores seeded.")


def do_backup(out):
    cfg = load_env()
    compose = compose_cmd()
    require_running_db(compose, cfg)
    path = Path(out or f"bauernkarte-{datetime.now():%Y%m%d-%H%M%S}.dump")
    # Custom format: compressed, and `pg_restore` can pick it apart.
    with path.open("wb") as dump:
        run(compose + ["exec", "-T", DB_SERVICE, "pg_dump", "-Fc",
                       "-U", cfg["DB_USER"], cfg["DB_NAME"]],
            stdout=dump, env=compose_env(cfg))
    click.echo(f"wrote {path} — restore with the command in service.md")


def do_cleanup(assume_yes):
    """Tear the whole compose stack down: containers, network, the built
    image and the `pgdata` volume."""
    cfg = load_env()
    compose = compose_cmd()
    if not assume_yes:
        click.confirm(
            "this removes the containers, the app image and the pgdata volume — "
            f"everything in the `{cfg['DB_NAME']}` database goes with it. "
            "Continue?",
            abort=True,
        )
    # `--rmi local` removes only the image built here, not the pulled
    # postgis one.
    run(compose + ["down", "--volumes", "--rmi", "local"],
        env=compose_env(cfg))
    click.echo("cleaned up — `./service.py up` rebuilds from scratch.")


@click.group()
def cli():
    """Bootstrap and deploy the containerized BauernKarte stack."""


@cli.command()
def env():
    """Create .env with generated secrets (never overwrites it)."""
    do_env()


@cli.command()
def db():
    """Start the `db` container and wait until it accepts connections."""
    do_db()


@cli.command()
def up():
    """Build and start the whole stack."""
    do_up()


@cli.command()
def down():
    """Stop and remove the containers (keeps data and image)."""
    do_down()


@cli.command()
def prepare():
    """Refresh the .sqlx/ offline query cache (needs Rust + sqlx-cli)."""
    do_prepare()


@cli.command()
@click.option("--demo", is_flag=True, help="Also add one fully filled-in demo shop.")
def stores(demo):
    """Seed OSM farm-shop/vending-machine data into the running db."""
    do_stores(demo)


@cli.command()
@click.option("--out", help="Output file (default: bauernkarte-<timestamp>.dump).")
def backup(out):
    """Dump the database (pg_dump custom format)."""
    do_backup(out)


@cli.command()
@click.option("--yes", is_flag=True, help="Skip the confirmation prompt.")
def cleanup(yes):
    """Remove the containers, the app image and the pgdata volume."""
    do_cleanup(yes)


if __name__ == "__main__":
    cli()
