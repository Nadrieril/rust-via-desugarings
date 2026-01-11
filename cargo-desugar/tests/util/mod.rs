//! Shared utility functions for use in tests.
//!
//! This is in `util/mod.rs` instead of `util.rs` to avoid cargo treating it like a test file.

#![allow(dead_code)]

use snapbox::{self, filter::Filter};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

#[derive(Clone, Copy)]
pub enum Action {
    Verify,
    Overwrite,
}

/// If `IN_CI=1`, check that the contents of `path` matches `output`, otherwise overwrite the file
/// with the given output.
pub fn compare_or_overwrite(output: impl AsRef<str>, path: &Path) -> snapbox::assert::Result<()> {
    let action = if std::env::var("IN_CI").as_deref() == Ok("1") {
        Action::Verify
    } else {
        Action::Overwrite
    };
    let output = strip_ansi_escapes::strip_str(output);
    let actual = snapbox::Data::text(output);
    let actual = snapbox::filter::FilterNewlines.filter(actual);
    match action {
        Action::Verify => expect_file_contents(path, actual)?,
        Action::Overwrite => actual.write_to_path(path)?,
    }
    Ok(())
}

/// Compare the file contents with the provided string and error with a diff if they differ.
fn expect_file_contents(path: &Path, actual: snapbox::Data) -> snapbox::assert::Result<()> {
    let expected = snapbox::Data::read_from(path, Some(snapbox::data::DataFormat::Text));
    let expected = snapbox::filter::FilterNewlines.filter(expected);

    if expected != actual {
        let mut buf = String::new();
        snapbox::report::write_diff(
            &mut buf,
            &expected,
            &actual,
            Some(&path.display()),
            Some(&"cargo-desugar output"),
            Default::default(),
        )
        .map_err(|e| e.to_string())?;
        Err(buf.into())
    } else {
        Ok(())
    }
}

/// Run rustfmt on captured output to stabilize diffs; falls back to the original output on error.
pub fn rustfmt_output(output: &str) -> String {
    let mut child = match Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return output.to_owned(),
    };
    if let Some(stdin) = child.stdin.as_mut() {
        let _ = stdin.write_all(output.as_bytes());
    }
    match child.wait_with_output() {
        Ok(result) if result.status.success() => {
            String::from_utf8(result.stdout).unwrap_or_else(|_| output.to_owned())
        }
        _ => output.to_owned(),
    }
}
