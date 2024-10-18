# hfb-auth
authentication for the horfimbor game

## development

### before first launch :

install Rust

```bash
cargo install cargo-watch
cargo install sqlx-cli --no-default-features --features rustls,mysql
```

### other launch : 

```bash
just start
just watch-server
```
###

build and push image to minikube : 

```shell
cargo sqlx prepare
eval $(minikube docker-env)
timestamp=$(date +%s)
docker build -t hfb-auth:${timestamp} .
```

### tools documentation : 

[sqlx-cli](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)

