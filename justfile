default:
    @just --list

# Auto-format the source tree
fmt:
    treefmt

# Run 'cargo run' on the project
run *ARGS:
    cargo run {{ARGS}}

watch:
    cargo watch -x r

# Build the app in release mode as statically linked binary with
# Nix, and then generate a FROM scratch docker image with the
# statically linked binary
container:
    nix build '.#container'

deploy:
    fly deploy --local-only
