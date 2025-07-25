# hfb-auth
authentication for the horfimbor game

## why not a simple Oauth 2 server ?

The main difference is that it allow to create multiple account on the distants services.

## development

### before first launch :

install Rust and then cargo watch :

```bash
cargo install cargo-watch
```

to run the commande in [justfile](./justfile) install [just](https://github.com/casey/just)

### start server for development : 

start the DB in docker compose :

```bash
just dc-start
```

start the server in watcher mode :

```bash
just watch
```
