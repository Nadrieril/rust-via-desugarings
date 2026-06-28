function toggle_railroad() {
    const grammarRailroad = get_railroad();
    set_railroad(!grammarRailroad);
    update_railroad();
}

function toggle_grammar_code() {
    const grammarCode = get_grammar_code();
    set_grammar_code(!grammarCode);
    update_grammar_code();
}

function show_railroad() {
    set_railroad(true);
    update_railroad();
}

function get_railroad() {
    let grammarRailroad = null;
    try {
        grammarRailroad = localStorage.getItem('grammar-railroad');
    } catch (e) {
    }
    return grammarRailroad === 'true';
}

function set_railroad(newValue) {
    try {
        localStorage.setItem('grammar-railroad', newValue);
    } catch (e) {
    }
}

function get_grammar_code() {
    let grammarCode = null;
    try {
        grammarCode = localStorage.getItem('grammar-code');
    } catch (e) {
    }
    return grammarCode !== 'false';
}

function set_grammar_code(newValue) {
    try {
        localStorage.setItem('grammar-code', newValue);
    } catch (e) {
    }
}

function update_railroad() {
    const grammarRailroad = get_railroad();
    const railroads = document.querySelectorAll('.grammar-railroad');
    railroads.forEach(element => {
        if (grammarRailroad) {
            element.classList.remove('grammar-hidden');
        } else {
            element.classList.add('grammar-hidden');
        }
    });
    const buttons = document.querySelectorAll('.grammar-toggle-railroad');
    buttons.forEach(button => {
        if (grammarRailroad) {
            button.innerText = "Hide syntax diagrams";
        } else {
            button.innerText = "Show syntax diagrams";
        }
    });
}

function update_grammar_code() {
    const grammarCode = get_grammar_code();
    const containers = document.querySelectorAll('.grammar-container');
    containers.forEach(element => {
        if (grammarCode) {
            element.classList.remove('grammar-code-hidden');
        } else {
            element.classList.add('grammar-code-hidden');
        }
    });
    const buttons = document.querySelectorAll('.grammar-toggle-code');
    buttons.forEach(button => {
        if (grammarCode) {
            button.innerText = "Hide code";
        } else {
            button.innerText = "Show code";
        }
    });
}

(function railroad_onload() {
    update_railroad();
    update_grammar_code();
})();
