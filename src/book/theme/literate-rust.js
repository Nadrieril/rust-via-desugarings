(function () {
    function slugForSource(sourcePath) {
        return sourcePath.replace(/[^A-Za-z0-9_-]+/g, "-").replace(/^-+|-+$/g, "");
    }

    function lineAnchorId(sourcePath, line) {
        return `literate-rust-${slugForSource(sourcePath)}-L${line}`;
    }

    function collectLinkData() {
        const links = new Map();
        document.querySelectorAll("script.literate-rust-links").forEach((script) => {
            let data;
            try {
                data = JSON.parse(script.textContent || "{}");
            } catch {
                return;
            }

            for (const [line, entries] of Object.entries(data)) {
                links.set(Number(line), entries);
            }
        });
        return links;
    }

    function nextCodeBlock(metadata) {
        let node = metadata.nextElementSibling;
        while (node && node.tagName !== "PRE") {
            node = node.nextElementSibling;
        }
        return node ? node.querySelector("code.language-rust") : null;
    }

    function lineStarts(text) {
        const starts = [0];
        for (let index = 0; index < text.length; index += 1) {
            if (text[index] === "\n" && index + 1 < text.length) {
                starts.push(index + 1);
            }
        }
        return starts;
    }

    function positionForOffset(root, targetOffset) {
        const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
        let offset = 0;
        let node;

        while ((node = walker.nextNode())) {
            const nextOffset = offset + node.textContent.length;
            if (targetOffset <= nextOffset) {
                return { node, offset: targetOffset - offset };
            }
            offset = nextOffset;
        }

        return null;
    }

    function insertLineAnchor(code, offset, id) {
        const position = positionForOffset(code, offset);
        if (!position || document.getElementById(id)) {
            return;
        }

        const marker = document.createElement("span");
        marker.id = id;
        marker.className = "literate-rust-line-anchor";

        const range = document.createRange();
        range.setStart(position.node, position.offset);
        range.collapse(true);
        range.insertNode(marker);
    }

    function wrapTextRange(code, start, end, href, text) {
        const currentText = code.textContent.slice(start, end);
        if (currentText !== text) {
            return;
        }

        const startPosition = positionForOffset(code, start);
        const endPosition = positionForOffset(code, end);
        if (!startPosition || !endPosition) {
            return;
        }

        const range = document.createRange();
        range.setStart(startPosition.node, startPosition.offset);
        range.setEnd(endPosition.node, endPosition.offset);

        const link = document.createElement("a");
        link.className = "literate-rust-def-link";
        link.href = href;

        try {
            range.surroundContents(link);
        } catch {
            return;
        }
    }

    function applyLinks(metadata, linksByLine) {
        const code = nextCodeBlock(metadata);
        if (!code) {
            return;
        }

        const sourcePath = metadata.dataset.sourcePath;
        const sourceLines = (metadata.dataset.sourceLines || "")
            .split(/\s+/)
            .filter(Boolean)
            .map(Number);
        if (!sourcePath || sourceLines.length === 0) {
            return;
        }

        const text = code.textContent;
        const starts = lineStarts(text);
        const operations = [];

        sourceLines.forEach((line, index) => {
            const start = starts[index];
            if (start === undefined) {
                return;
            }

            insertLineAnchor(code, start, lineAnchorId(sourcePath, line));

            for (const entry of linksByLine.get(line) || []) {
                operations.push({
                    start: start + entry.start,
                    end: start + entry.end,
                    href: entry.href,
                    text: entry.text,
                });
            }
        });

        operations
            .sort((left, right) => right.start - left.start || right.end - left.end)
            .forEach((operation) => {
                wrapTextRange(
                    code,
                    operation.start,
                    operation.end,
                    operation.href,
                    operation.text,
                );
            });
    }

    function scrollToCurrentHash() {
        if (!window.location.hash) {
            return;
        }

        const id = decodeURIComponent(window.location.hash.slice(1));
        const target = document.getElementById(id);
        if (target) {
            target.scrollIntoView();
        }
    }

    window.addEventListener("load", () => {
        const linksByLine = collectLinkData();
        document.querySelectorAll(".literate-rust-source").forEach((metadata) => {
            applyLinks(metadata, linksByLine);
        });
        scrollToCurrentHash();
    });
})();
