set shell := ["bash", "-uc"]
set dotenv-load

alias dc-start := dc-up
dc-up *SRV :
    docker compose up -d --build --force-recreate --remove-orphans {{SRV}}

dc-up-db:
    just dc-up kurrentdb redis

dc-up-log *SRV :
    just dc-up {{SRV}}
    docker compose logs --follow {{SRV}}

alias dc-stop := dc-down
dc-down:
    docker compose down --remove-orphans

dc-reset:
    just dc-down
    just dc-up

alias ff := open
open:
    firefox $APP_HOST

watch:
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

