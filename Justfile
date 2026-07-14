build-interactive-wasm:
    cargo build --release --lib --target wasm32-unknown-unknown
    wasm-bindgen --target web --out-dir src/book/theme/interactive-desugar-wasm --out-name interactive_desugar_wasm target/wasm32-unknown-unknown/release/rust_via_desugarings.wasm

serve:
    cd src && mdbook serve
