set shell := ["bash", "-uc"]
set dotenv-load

alias dc-up := dc-start
dc-start:
    docker compose up -d --build --force-recreate {{SRV}}
    docker compose logs --follow {{SRV}}

alias dc-down := dc-stop
dc-stop:
    docker compose down --remove-orphans

dc-reset:
    just dc-down
    just dc-start

alias ff := open
open:
    firefox $APP_HOST

watch: dc-start
    export $(grep -v '^#' .env | xargs) && bacon run-long

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

