#!/usr/bin/env python3
"""mdBook preprocessor for literate Rust programming.

Within a `.md.rs` file, any `//@` comment indicates a markdown block. After processing,
such blocks of text become the main markdown source, and any rust code found in between
such blocks is put inside a code block.

Moreover, any line that ends in `//#` will be hidden in the output.
"""

from __future__ import annotations

import json
import re
import sys
from typing import Any, Dict, Iterable, List

COMMENT_LINE = re.compile(r"^([ \t]*)//@ ?(.*)$")
HIDDEN_CODE_MARKER = re.compile(r"^(.*?)(?:\s*//#)$")


def literate_rust_to_markdown(content: str) -> str:
    result: List[str] = []
    code_buffer: List[str] = []

    def flush_code() -> None:
        if not code_buffer:
            return
        result.append("```rust")
        result.extend(code_buffer)
        result.append("```")
        code_buffer.clear()

    for line in content.splitlines():
        match = COMMENT_LINE.match(line)
        if match:
            flush_code()
            line = match.group(2)
            result.append(line)
        elif not HIDDEN_CODE_MARKER.match(line.rstrip()):
            code_buffer.append(line)

    flush_code()

    output = "\n".join(result)
    output += "\n"
    return output

def is_literate_chapter(chapter: Dict[str, Any]) -> bool:
    path = chapter.get("path") or ""
    source = chapter.get("source_path") or ""
    return path.endswith(".md.rs") or source.endswith(".md.rs")


def process_section(section: Dict[str, Any]) -> None:
    """Recursively walk mdBook sections."""
    if "Chapter" in section:
        chapter = section["Chapter"]
        if is_literate_chapter(chapter):
            chapter["content"] = literate_rust_to_markdown(chapter.get("content", ""))
        for child in chapter.get("sub_items", []):
            process_section(child)
    elif "PartTitle" in section:
        for child in section["PartTitle"].get("sub_items", []):
            process_section(child)

def process_book(sections: Iterable[Dict[str, Any]]) -> None:
    for section in sections:
        process_section(section)

def main(argv: List[str]) -> None:
    if len(argv) >= 2 and argv[1] == "supports":
        return

    [_, book] = json.load(sys.stdin)
    process_book(book.get("sections", []))
    json.dump(book, sys.stdout)

if __name__ == "__main__":
    main(sys.argv)
