default: watch

startup:
    docker compose up -d
    sqlx database create

reset-db: startup
    sqlx database reset

watch: sqlx-up
    export $(grep -v '^#' .env | xargs) && \
            cargo watch -w src -w templates -w web  \
            -x "run"

precommit:
    cargo fmt
    cargo clippy
    cargo sqlx prepare

new-migration name:
    sqlx migrate add -r {{name}}

[private]
sqlx-up:
    sqlx database create

[private]
sqlx-down:
    sqlx database drop
