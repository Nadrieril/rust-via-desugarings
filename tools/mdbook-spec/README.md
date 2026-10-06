# mdbook-spec

mdbook extension for our spec-writing.

This crate provides two preprocessors.

`mdbook-spec` (`[preprocessor.spec]`) runs for all renderers. It:
- makes sure the interactive desugaring wasm is up to date
- renders literate Rust (`.md.rs`) chapters as markdown:
  `//@` lines become markdown, lines ending in `//#` are hidden,
  `interactive_example!` invocations become interactive examples,
  and the remaining lines become code blocks;
- adds go-to-definition links to that code,
  using `cargo doc` with `--generate-link-to-definition`;
- adds a "▶️ " prefix to the titles of literate Rust chapters;
- renders `grammar` blocks;
- turns Reference rule links into links to the Reference (see below).

`mdbook-html-filenames` (`[preprocessor.html-filenames]`)
runs only for the HTML renderer. It makes the output filenames for Literate Rust files cleaner (see below).

## Reference Rule Links

This:
```markdown
[ref:associated.fn.method.self-pat-shorthands]
```
gets turned into a link to the appropriate Reference section.

The Reference doesn't seem to expose a mapping from anchor
to relevant page URL, so we maintain a manual one.
Add entries to the table if you use a section we haven't referenced before.

## Literate Rust chapter filenames

By default mdBook would render a chapter with path `foo.md.rs` as `foo.md.html`, but it would render markdown links to `foo.md.rs` as links to `foo.html`.

The `mdbook-html-filenames` preprocessor arranges for such a chapter to be rendered as `foo.html` instead, by changing its `path` field in the mdBook data.

Markdown content should write links to these chapters using the source filename, `foo.md.rs`.

NB mdBook drops the fragment from a link like `foo.md.rs#bar`,
so links to fragments in these chapters don't work at present.
