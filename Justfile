serve:
    cd src && mdbook serve

build:
    cd cargo-desugar && cargo build

test: build
    ./scripts/run_rustc_tests.sh
