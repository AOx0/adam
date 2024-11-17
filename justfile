run iface='lo': build-firewall
    env ADAM_FIREWALL_IFACE='{{iface}}' zellij --layout .zellij/run.kdl

run-simple iface='lo': build-firewall
    env ADAM_FIREWALL_IFACE='{{iface}}' zellij --layout .zellij/simple.kdl

build-firewall:
    RUST_LOG=info cargo xtask build firewall
    RUST_LOG=info cargo xtask build firewall --release

run-firewall iface='lo':
    RUST_LOG=info cargo xtask run firewall --release -- -i {{iface}}

run-controller:
    cargo build --release && sudo ./target/release/controller

run-front:
    cargo run --bin front
    
run-front-watch:
    cargo watch -cqs "cargo run --bin front"
