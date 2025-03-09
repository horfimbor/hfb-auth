set shell := ["bash", "-uc"]
set dotenv-load

alias dc-up := dc-start
dc-start:
    docker compose up -d
    firefox $APP_HOST

alias dc-down := dc-stop
dc-stop:
    docker compose down

dc-reset:
    docker compose down -v
    just dc-start


open:
    firefox $APP_HOST

watch:
    export $(grep -v '^#' .env | xargs) && \
            cargo watch -w server -w shared  \
            -x "run -p hfb-auth-server service"

add-admin uuid:
    cargo run -p hfb-auth-server -- user-update --user {{uuid}} --role Admin

reset-password uuid new_password:
    cargo run -p hfb-auth-server -- user-update --user {{uuid}} --password {{new_password}}

precommit:
    cargo fmt
    cargo clippy -- -D clippy::expect_used -D clippy::panic -D clippy::unwrap_used
    cargo test


deploy-local:
    eval $(minikube docker-env) && \
      timestamp=$(date +%s) && \
      docker build -t hfb-auth:${timestamp} . && \
      sed -i "s/hfb-auth:[0-9]*/hfb-auth:${timestamp}/" $LOCAL_K8S/apps/hfb-auth/deployment.yaml

