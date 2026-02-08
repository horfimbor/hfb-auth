# hfb-auth
authentication for the horfimbor game

## why not a simple Oauth 2 server ?

The main difference is that it allow to create multiple account on the distants services.

## development

### before first launch :

install [Rust](https://rust-lang.org/) with [rustup](https://rustup.rs/) and then [bacon](https://dystroy.org/bacon/) :

```bash
cargo install bacon
```

to run the commands in [justfile](./justfile) install [just](https://github.com/casey/just)

### start server for development : 

start the DB in docker compose :

```bash
just dc-up
```

start the server in watcher mode :

```bash
just watch
```
