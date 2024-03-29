default: watch

startup:
    docker compose up -d

cleanup:
    docker compose down

reset-db: cleanup startup

watch: sqlx-up
    export $(grep -v '^#' .env | xargs) && \
            cargo watch -w src -w templates -w web  \
            -x "run"

precommit:
    cargo fmt
    cargo clippy


new-migration name:
    sqlx migrate add -r {{name}}

[private]
bdd-up:
    docker compose up -d

[private]
sqlx-up:
    sqlx database create
    sqlx migrate run

[private]
sqlx-down:
    sqlx database drop
