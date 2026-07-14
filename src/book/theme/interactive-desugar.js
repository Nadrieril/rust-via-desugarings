(function () {
    const scriptUrl = document.currentScript ? document.currentScript.src : null;
    const wasmModulePath = "../../theme/interactive-desugar-wasm/interactive_desugar_wasm.js";
    const wasmBinaryPath = "../../theme/interactive-desugar-wasm/interactive_desugar_wasm_bg.wasm";
    let wasmModulePromise = null;

    function assetUrl(path) {
        return scriptUrl ? new URL(path, scriptUrl).href : path;
    }

    function loadWasmModule() {
        if (!wasmModulePromise) {
            wasmModulePromise = import(assetUrl(wasmModulePath)).then(async (module) => {
                if (typeof module.default === "function") {
                    await module.default(assetUrl(wasmBinaryPath));
                }
                if (typeof module.interactive_desugar_example !== "function") {
                    throw new Error("interactive_desugar_example wasm export not found");
                }
                return module;
            });
        }
        return wasmModulePromise;
    }

    function unavailableText(error) {
        const detail = error && error.message ? error.message : String(error);
        return [
            "Interactive desugarings are unavailable.",
            "Rebuild the book; mdbook-spec regenerates these assets when wasm-bindgen is available.",
            "",
            detail,
        ].join("\n");
    }

    function thrownText(error) {
        return error && error.message ? error.message : String(error);
    }

    function setOutput(output, text, isError) {
        output.textContent = text;
        output.parentElement.classList.toggle("is-error", isError);
    }

    function readStoredInput(storageKey) {
        try {
            return window.localStorage.getItem(storageKey);
        } catch {
            return null;
        }
    }

    function writeStoredInput(storageKey, input) {
        try {
            window.localStorage.setItem(storageKey, input);
        } catch {
        }
    }

    function setupInteractiveDesugar(root) {
        const exampleId = root.dataset.desugarExample;
        const editor = root.querySelector(".interactive-desugar__editor");
        const desugarOutput = root.querySelector(".interactive-desugar__output code");
        if (!exampleId || !editor || !desugarOutput) {
            return;
        }

        const storageKey = `interactive-desugar:${exampleId}`;
        const savedInput = readStoredInput(storageKey);
        if (savedInput !== null) {
            editor.value = savedInput;
        }

        let runId = 0;
        let timeout = null;

        async function run() {
            const currentRun = ++runId;
            setOutput(desugarOutput, "Running...", false);

            let wasmModule;
            try {
                wasmModule = await loadWasmModule();
            } catch (error) {
                if (currentRun === runId) {
                    setOutput(desugarOutput, unavailableText(error), true);
                }
                return;
            }

            try {
                const rendered = wasmModule.interactive_desugar_example(exampleId, editor.value);
                if (currentRun === runId) {
                    setOutput(desugarOutput, rendered, false);
                }
            } catch (error) {
                if (currentRun === runId) {
                    setOutput(desugarOutput, thrownText(error), true);
                }
            }
        }

        function scheduleRun() {
            writeStoredInput(storageKey, editor.value);
            window.clearTimeout(timeout);
            timeout = window.setTimeout(run, 180);
        }

        editor.addEventListener("input", scheduleRun);
        run();
    }

    function setupAllInteractiveDesugarings() {
        document
            .querySelectorAll(".interactive-desugar")
            .forEach(setupInteractiveDesugar);
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", setupAllInteractiveDesugarings);
    } else {
        setupAllInteractiveDesugarings();
    }
})();
