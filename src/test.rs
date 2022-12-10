use crate::builtin_citeproc_support;
use std::path::{Path, PathBuf};
use std::process::Command;

const EXPECTED_INLINE: &str = "Paragraph contents with an inline [1, p. 22] reference.";

const EXPECTED_REFERENCE: &str = "The Evidence for an Autumnal New Year in Pre-Exilic Israel
Reconsidered";

#[test]
fn test_book() {
    println!(
        "Using builtin citeproc support: {}",
        builtin_citeproc_support().unwrap()
    );
    let test_book = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-book");
    mdbook_clean(&test_book);
    let output = Command::new("mdbook")
        .arg("build")
        .current_dir(&test_book)
        .output()
        .expect("Failed to call mdbook - is it installed?");
    let std_err = std::str::from_utf8(&output.stderr).unwrap();
    if !output.status.success() {
        panic!("mdbook failed to execute: {}", std_err);
    }
    if std_err.contains("error") {
        panic!("mdbook errors: {}", std_err);
    }
    let output_chapter = std::fs::read_to_string(test_book.join("book").join("chapter_1.html"))
        .expect("Failed to read chapter_1.html");
    let output_chapter = line_break_to_space(&output_chapter);
    assert!(output_chapter.contains(&line_break_to_space(EXPECTED_INLINE)));
    assert!(output_chapter.contains(&line_break_to_space(EXPECTED_REFERENCE)));
}

fn line_break_to_space(s: &str) -> String {
    let line_strings: Vec<&str> = s.lines().collect();
    line_strings.join(" ")
}

fn mdbook_clean(dir: &Path) {
    let output = Command::new("mdbook")
        .arg("clean")
        .current_dir(dir)
        .output()
        .expect("Failed to call mdbook - is it installed?");
    if !output.status.success() {
        let std_err = std::str::from_utf8(&output.stderr).unwrap();
        panic!("mdbook failed to clean: {}", std_err);
    }
}
