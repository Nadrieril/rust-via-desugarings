# Tests

## UI tests

Each UI test is defined by a `.rs` file under `tests/ui/` containing a test program.

To run the UI tests:

    cargo test --test ui

Add a test name (or part of one) to run only matching tests.

### Output files

Running a test writes one or more of the following files in the same directory as the test:

- `<name>.desugared.rs`: the desugared program, if desugaring succeeded
- `<name>.stderr`: the error message, if processing reported an error
- `<name>.out`: the program's output, if it was run successfully

Any of these files that the run doesn't write is deleted.

(Tests runs that fail because of an invalid directive or a failed printing round-trip check are
exceptional, and don't modify any output files.)

These files are committed.

The test harness doesn't compare them against a 'blessed' version. That means a change in output
doesn't make a test fail.

(You can review changes with `git diff` instead. This is like always running rustc's test suite with
`--bless`.)

### Parsing and desugaring

When a test is run the program in its `.rs` file is parsed, and if it parses successfully all the
desugarings are applied.

If parsing or desugaring reports an error, the `.stderr` file is written. Otherwise the
`.desugared.rs` file is written.

Ordinarily the test fails if parsing or the desugaring process reports an error; use
the `expect-rejection` directive (see below) to modify this behaviour.

### Runnable tests

If the `run` directive is present and desugaring succeeds, the desugared program is checked using
a-mir-formality and run using MiniRust.

If the program runs successfully, the `.out` file is written. Otherwise the `.stderr` file is
written.

Ordinarily the test fails if the program doesn't pass the checks or doesn't run successfully; use
the `expect-rejection` directive (see below) to modify this behaviour.

### Directives

Directives are lines starting with `//@` (optionally preceded by whitespace). They may appear
anywhere in a test file.

There's no syntax for adding remarks after a directive.

An unknown or duplicate directive makes the test fail.

#### `run`

`//@ run` marks tests which should be checked and run as described above.

#### `expect-rejection`

`//@ expect-rejection: <expectation>` marks tests for which processing the program is expected to
report a `ProgramRejection`. A test marked in this way fails if processing succeeds or reports a
rejection which doesn't match the expectation.

A `ProgramRejection` records which stage of the pipeline rejected the program, and what kind of
rejection it was. The expectation can specify either or both of these, in one of the following
forms:

- `<Kind>`
- `<Kind> @ <Stage>`
- `@ <Stage>`

The kinds are `Invalid`, `Unimplemented`, `InternalError`, `RuntimeError`, and `Unclassified`;
see the Rustdoc for `RejectionKind` in `src/lib.rs` for what each one means.

The stages are `Parse`, `Desugaring`, `FormalityTranslation`, `FormalityCheck`,
`MiniRustTranslation`, and `MiniRustExecution`; see the Rustdoc for `Stage` in `src/lib.rs`.

For example:

```rust
//@ expect-rejection: Invalid @ Desugaring
fn foo(x: bool, self) {}
```

#### `known-bug`

`//@ known-bug: <text>` marks tests whose current behaviour is known to be wrong. The text is a
place to put a pointer to more information (eg an issue number).

When this directive is present the other directives and the committed output files record what
currently happens, not what should happen.

`known-bug` doesn't have any effect on the circumstances in which the test passes or on the written
output files.

For example:

```rust
//@ known-bug: #123
//@ expect-rejection: Unimplemented
// This should be accepted.
fn f() { ... }
```

### Printing round-trip check

If a test program parses successfully, it is then printed internally and re-parsed to check that the
printer behaves correctly.

If the re-parsed form isn't equivalent to the parsed form, the test fails (the output files aren't
modified, and the `expect-rejection` directive is ignored).
