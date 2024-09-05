set shell := ["bash", "-uc"]
set dotenv-load

startup:
    docker compose up -d
    sleep 5
    sqlx database create

stop:
    docker compose down

reset-db: startup
    sqlx database reset

watch-server: sqlx-up
    export $(grep -v '^#' .env | xargs) && \
            cargo watch -w server -w shared  \
            -x "run -p hfb-auth-server"

add-admin key:
    export $(grep -v '^#' .env | xargs) && cargo run -p hfb-auth-server -- --add-admin {{key}}

precommit:
    cargo fmt
    cargo clippy
    cargo sqlx prepare

new-migration name:
    sqlx migrate add -r {{name}}

deploy-local: sqlx-up
    cargo sqlx prepare
    eval $(minikube docker-env) && \
      timestamp=$(date +%s) && \
      docker build -t hfb-auth:${timestamp} . && \
      sed -i "s/hfb-auth:[0-9]*/hfb-auth:${timestamp}/" $LOCAL_K8S/apps/hfb-auth/deployment.yaml

[private]
sqlx-up:
    sqlx database create
    sqlx migrate run

[private]
sqlx-down:
    sqlx database drop
