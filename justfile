[positional-arguments]
run iface='lo' flags='':
    env ADAM_FIREWALL_IFACE='{{iface}}' zellij --layout '{{ if flags =~ ".*--release.*" { ".zellij/run_release.kdl" } else { ".zellij/run.kdl" } }}'

# Run backend & controllers
[positional-arguments]
run-simple iface='lo' flags='':
    env ADAM_FIREWALL_IFACE='{{iface}}' zellij --layout {{ if flags =~ ".*--release.*" { ".zellij/simple_release.kdl" } else { ".zellij/simple.kdl" } }}

# Run backend
[positional-arguments]
run-controller flags='': (build-controller flags)
    sudo RUST_LOG=info {{ if flags =~ ".*--release.*" { "./target/release/controller" } else { "./target/debug/controller" } }}

# Run frontend
[positional-arguments]
run-front flags='': (build-front flags)
    {{ if flags =~ ".*--release.*" { "./target/release/front" } else { "./target/debug/front" } }}

# Run controllers    
[positional-arguments]
run-firewall iface='lo' flags='': (build-firewall flags)
    sudo RUST_LOG=info {{ if flags =~ ".*--release.*" { "./target/release/firewall" } else { "./target/debug/firewall" } }} -i {{iface}}

# Build
[positional-arguments]
build flags='': (build-firewall flags) (build-controller flags) (build-front flags) 

## Build frontend
[positional-arguments]
build-front flags='': build-tailwind
    cd ./front/ && cargo build $@

build-tailwind:
    tailwindcss -i ./front/static/input.css -o ./front/static/styles.css -c ./tailwind.config.js 

## Build backend
[positional-arguments]
build-controller flags='':
    cd ./controller && cargo build $@

## Build controllers
[positional-arguments]
build-firewall flags='':
    RUST_LOG=info cargo xtask build firewall $@
