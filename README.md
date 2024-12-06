# adam

## Prerequisites

### Cranelift Codegen Backend

```sh
rustup component add rustc-codegen-cranelift-preview --toolchain nightly
```

### Dependencies

1. Install [`lld`](https://lld.llvm.org/): `paru -S lld`; `sudo apti-get install lld`
2. Install [`bpf-linker`](https://github.com/aya-rs/bpf-linker): `cargo install bpf-linker`
3. Install [`zellij`](https://zellij.dev/): `cargo install zellij`
4. Install [`cargo-watch`](https://github.com/watchexec/cargo-watch): `cargo install cargo-watch`
5. Install [`just`](https://github.com/casey/just): `cargo install just`
6. Install [`hurl`](https://hurl.dev/): `cargo install hurl`

You may install all packages via your package manager, for example, for Arch Linux:

```sh
paru -S just hurl zellij cargo-watch lld
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

## Documentation

For detailed instructions on how to run the code, please refer to the [Latex documentation](docs/usage_guide.tex).
