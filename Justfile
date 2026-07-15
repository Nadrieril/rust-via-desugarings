build-interactive-wasm profile="release":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "{{profile}}" = release ]; then
        cargo build --release --lib --target wasm32-unknown-unknown
    else
        cargo build --lib --target wasm32-unknown-unknown
    fi
    artifact="target/wasm32-unknown-unknown/{{profile}}/rust_via_desugarings.wasm"
    wasm_output="src/book/theme/interactive-desugar-wasm/interactive_desugar_wasm_bg.wasm"
    js_output="src/book/theme/interactive-desugar-wasm/interactive_desugar_wasm.js"
    if [ ! -e "$wasm_output" ] || [ ! -e "$js_output" ] || [ "$artifact" -nt "$wasm_output" ] || [ "$artifact" -nt "$js_output" ]; then
        wasm-bindgen --target web --out-dir src/book/theme/interactive-desugar-wasm --out-name interactive_desugar_wasm "$artifact"
    fi

serve:
    cd src && mdbook serve
