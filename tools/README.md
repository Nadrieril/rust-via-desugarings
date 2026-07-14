# Tooling

This directory contains tooling for:
- The custom grammar syntax we made up, namely:
    - Parsing it;
    - Turning it into rustylr syntax to make a real parser;
    - Displaying it nicely, and generating pretty grammar diagrams just like in the Reference;
- The literate Rust syntax we made up, namely:
    - Turning it into markdown;
    - Go-to-definition using rustdoc;
    - Interactive desugaring examples powered by wasm.

Everything in this directory is LLM-authored. When the book matures we should clean it up but for
now I'm iterating freely on convenient user-facing features.

The only part of this tooling that is trusted is the grammar->rustylr translation, as this could
lead to incorrect parses. I'll also note that rustylr itself appears to now be quite vibecoded. Once
we have all the complex corner cases of the grammar working (like `let else` or raw string
literals), we should reconsider the parser generator pipeline.
