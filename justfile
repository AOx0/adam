run iface='lo':
    env ADAM_FIREWALL_IFACE='{{iface}}' zellij --layout .zellij/run.kdl

run-simple iface='lo':
    env ADAM_FIREWALL_IFACE='{{iface}}' zellij --layout .zellij/simple.kdl

run-firewall iface='lo':
    RUST_LOG=info cargo xtask run firewall --release -- -i {{iface}}

run-controller:
    cargo build --release && sudo ./target/release/controller

run-front:
    cargo run --release --bin front
    
run-front-watch:
    cargo watch -cqs "cargo run --release --bin front"
