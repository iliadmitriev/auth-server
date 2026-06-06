# Auth Server

Authentication microservice built with Rust.

## Overview

Handles user registration, sign-in, access token refresh, and sign-out. Issues
JWT access tokens (HS256) and stores refresh sessions in Redis. Passwords are
hashed with Argon2.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Runtime | Rust + Tokio |
| Framework | Axum 0.8 |
| Database | PostgreSQL (SQLx) |
| Cache & Sessions | Redis |
| Password hashing | Argon2 |
| Tokens | JWT (HS256) |

## Prerequisites

- [Rust](https://rustup.rs) (stable)
- [Docker](https://www.docker.com) (for local PostgreSQL and Redis)

## Quick Start

```bash
git clone <repo-url>
cd auth-server

# Start PostgreSQL and Redis
docker compose up -d

# Install SQLx CLI and run migrations
cargo install sqlx-cli
sqlx migrate run

# Run the server
cargo run
```

The server listens on `http://127.0.0.1:8080`.

## Configuration

Settings are loaded from `config.yaml` and overridable via environment variables
prefixed with `AUTH_SERVER__` (powered by [Figment](https://crates.io/crates/figment)).

| Setting | Default | Description |
|---------|---------|-------------|
| `server.host` | `127.0.0.1` | Bind address |
| `server.port` | `8080` | Bind port |
| `server.log_format` | `pretty` | `pretty` or `json` |
| `database.url` | `postgres://auth:secret@localhost:5432/auth` | PostgreSQL connection string |
| `database.max_connections` | `10` | Connection pool size |
| `redis.url` | `redis://localhost:6379/0` | Redis connection string |
| `jwt.secret` | `secret` | HMAC secret for signing access tokens |
| `jwt.access_token_duration_minutes` | `15` | Access token TTL |
| `jwt.refresh_token_duration_days` | `7` | Refresh session TTL |
| `smtp.host` | `127.0.0.1` | SMTP server host |
| `smtp.port` | `1025` | SMTP server port |
| `smtp.from` | `test@example.com` | From address for emails |

## API

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/signup` | Create a new account |
| `POST` | `/signin` | Sign in and receive tokens |
| `POST` | `/refresh` | Refresh an access token |
| `POST` | `/signout` | Invalidate a refresh session |
| `GET`  | `/health` | Health check |
| `GET`  | `/metrics` | Prometheus metrics |

### Usage

```bash
# Sign up
curl -X POST http://127.0.0.1:8080/signup \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "secret123"}'
# 201 {"message":"user created successfully. Please check your email for verification.","email":"user@example.com"}

# Sign in
curl -X POST http://127.0.0.1:8080/signin \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "secret123"}'
# 200 {"access_token":"eyJ...","refresh_token":"...","token_type":"Bearer"}

# Refresh access token
curl -X POST http://127.0.0.1:8080/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "..."}'
# 200 {"access_token":"eyJ...","token_type":"Bearer"}

# Sign out
curl -X POST http://127.0.0.1:8080/signout \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "..."}'
# 200

# Health check
curl http://127.0.0.1:8080/health
# 200 {"status":"healthy","message":"Auth server is running"}
```

## Database

```bash
cargo install sqlx-cli

# Apply pending migrations
sqlx migrate run

# Revert the last migration
sqlx migrate revert
```

### Schema

```sql
create table users (
    id            UUID         primary key default gen_random_uuid(),
    email         VARCHAR(255) unique      not null,
    password_hash TEXT                     not null,
    is_verified   BOOLEAN                  not null default false,
    created_at    TIMESTAMPTZ              not null default now()
);
```

## Development

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings
sqruff lint .
cargo audit
```

Pre-commit hooks are configured to run formatting, linting, and SQL checks
automatically:

```bash
pip install pre-commit
pre-commit install
```
