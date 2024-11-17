# adam

## Prerequisites

1. Install [`bpf-linker`](https://github.com/aya-rs/bpf-linker): `cargo install bpf-linker`
2. Install [`zellij`](https://zellij.dev/): `cargo install zellij`
3. Install [`cargo-watch`](https://github.com/watchexec/cargo-watch): `cargo install cargo-watch`
4. Install [`just`](https://github.com/casey/just): `cargo install just`
5. Install [`hurl`](https://hurl.dev/): `cargo install hurl`

You may install all packages via your package manager, for example, for Arch Linux:

```sh
paru -S just hurl zellij cargo-watch
cargo install bpf-linker
```

## Run

All recipe definitions are available at the [`justfile`](https://github.com/AOx0/adam/blob/main/justfile).

### Everything

To run all components, execute:

```sh
just run
```

You may also specify the firewall wifi interface you want to attach to:

```sh
just run wlan0
```
### Backend

To run up to the `controller` perform a:

```sh
just run-simple
```

### Frontend

To run the frontend perform a:

```sh
just run-front-watch
```
