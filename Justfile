build-interactive-wasm profile="release":
    if [ "{{profile}}" = release ]; then cargo build --release --lib --target wasm32-unknown-unknown; else cargo build --lib --target wasm32-unknown-unknown; fi
    wasm-bindgen --target web --out-dir src/book/theme/interactive-desugar-wasm --out-name interactive_desugar_wasm target/wasm32-unknown-unknown/{{profile}}/rust_via_desugarings.wasm

serve:
    cd src && mdbook serve
