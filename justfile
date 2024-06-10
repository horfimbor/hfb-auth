default: watch

set shell := ["bash", "-uc"]
set dotenv-load

start:
    docker compose up -d
    sleep 5
    sqlx database create

stop:
    docker compose down

reset-db: start
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
