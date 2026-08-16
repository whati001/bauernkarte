#!/usr/bin/env python3
"""Bootstrap the local (non-container) dev environment: a `.env` file,
a podman Postgres+PostGIS container, and migrations run against it.

See bootstrap.md for details and prerequisites.

This is the "run cargo directly on the host" path. For a fully
containerized stack (app + db, no local Rust toolchain needed), use
`docker compose up --build` instead — see docker-compose.yml.

Usage:
    python3 bootstrap.py
"""

import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent
ENV_FILE = REPO_ROOT / ".env"
ENV_EXAMPLE = REPO_ROOT / ".env.example"

CONTAINER_NAME = "product_finder_db"
DB_IMAGE = "docker.io/postgis/postgis:16-3.4"
DB_USER = "product_finder"
DB_PASSWORD = "dev"
DB_NAME = "product_finder"
# Host networking (rootless podman, no docker daemon in this env) means
# this port is the *host's* port, so it's set away from Postgres's
# default 5432 to avoid clashing with any other local Postgres.
DB_PORT = "5433"

CARGO_BIN = Path.home() / ".cargo" / "bin"


def run(cmd, **kwargs):
    print(f"$ {' '.join(cmd)}")
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
        sys.exit(f"error: `{cmd_name}` not found on PATH. {hint}")


def ensure_env_file():
    if ENV_FILE.exists():
        print(f"{ENV_FILE.name} already exists, leaving it alone")
        return
    if not ENV_EXAMPLE.exists():
        sys.exit(f"error: {ENV_EXAMPLE.name} is missing, can't seed .env")
    shutil.copyfile(ENV_EXAMPLE, ENV_FILE)
    # .env.example ships the production-shaped defaults (port 5432,
    # SECURE_COOKIES=true); local dev needs this container's port and
    # no TLS requirement on the session cookie (see README).
    text = ENV_FILE.read_text()
    text = text.replace(":5432/", f":{DB_PORT}/").replace(
        "SECURE_COOKIES=true", "SECURE_COOKIES=false"
    )
    ENV_FILE.write_text(text)
    print(f"wrote {ENV_FILE.name} from {ENV_EXAMPLE.name} (port -> {DB_PORT}, "
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
            print(f"{CONTAINER_NAME} already running")
        else:
            print(f"{CONTAINER_NAME} exists but is stopped, starting it")
            run(["podman", "start", CONTAINER_NAME])
        return
    print(f"creating {CONTAINER_NAME} ({DB_IMAGE})")
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
    print("waiting for postgres to accept connections...")
    deadline = time.time() + timeout
    while time.time() < deadline:
        result = subprocess.run(
            ["podman", "exec", CONTAINER_NAME, "pg_isready",
             "-U", DB_USER, "-d", DB_NAME, "-p", DB_PORT],
            capture_output=True,
        )
        if result.returncode == 0:
            print("postgres is ready")
            return
        time.sleep(1)
    sys.exit(f"error: postgres did not become ready within {timeout}s "
              f"(check `podman logs {CONTAINER_NAME}`)")


def database_url_from_env_file():
    for line in ENV_FILE.read_text().splitlines():
        if line.startswith("DATABASE_URL="):
            return line.split("=", 1)[1]
    sys.exit(f"error: DATABASE_URL not set in {ENV_FILE.name}")


def run_migrations(env):
    env = {**env, "DATABASE_URL": database_url_from_env_file()}
    run(["sqlx", "migrate", "run"], cwd=REPO_ROOT, env=env)


def main():
    require("podman", None,
             "install podman, or adapt DB_IMAGE/run args in this script for docker.")
    ensure_env_file()
    ensure_container()
    wait_for_db()

    env = cargo_env()
    require("sqlx", env["PATH"],
             "install it with: cargo install sqlx-cli --no-default-features --features postgres")
    run_migrations(env)

    print()
    print("bootstrap complete. Next:")
    print(f"  export PATH=\"{CARGO_BIN}:$PATH\"   # if cargo isn't already on PATH")
    print("  cargo run")


if __name__ == "__main__":
    main()
