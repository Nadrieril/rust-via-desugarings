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
