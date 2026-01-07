serve:
    cd book && mdbook serve

build:
    cargo build

test: build
    ./scripts/run_rustc_tests.sh
