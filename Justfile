serve:
    cd src && mdbook serve

build:
    cargo build -p cargo-desugar

test-ui:
    cargo test -p cargo-desugar --test ui

test-rustc:
    cargo test -p cargo-desugar --test rustc -- -q
