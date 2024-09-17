run-firewall iface='lo':
    RUST_LOG=info cargo xtask run firewall --release -- -i {{iface}}

run-controller:
    cargo build --release && sudo ./target/release/controller
