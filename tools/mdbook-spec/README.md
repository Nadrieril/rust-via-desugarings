# mdbook-spec

mdbook extension for our spec-writing.

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
