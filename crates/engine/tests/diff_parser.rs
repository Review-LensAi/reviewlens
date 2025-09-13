use engine::diff_parser;

#[test]
fn parse_empty_diff_returns_no_files() {
    let files = diff_parser::parse("").expect("should parse");
    assert!(files.is_empty());
}

#[test]
fn parse_simple_unified_diff() {
    use engine::diff_parser::Line;

    let diff = r#"diff --git a/foo.txt b/foo.txt
--- a/foo.txt
+++ b/foo.txt
@@ -1,2 +1,3 @@
 line1
-line2
+line2modified
+line3
"#;

    let files = diff_parser::parse(diff).expect("should parse");
    assert_eq!(files.len(), 1);

    let file = &files[0];
    assert_eq!(file.path, "foo.txt");
    assert_eq!(file.hunks.len(), 1);

    let hunk = &file.hunks[0];
    assert_eq!(hunk.old_start, 1);
    assert_eq!(hunk.old_lines, 2);
    assert_eq!(hunk.new_start, 1);
    assert_eq!(hunk.new_lines, 3);
    assert_eq!(hunk.lines.len(), 4);

    assert!(matches!(&hunk.lines[0], Line::Context(line) if line == "line1"));
    assert!(matches!(&hunk.lines[1], Line::Removed(line) if line == "line2"));
    assert!(matches!(&hunk.lines[2], Line::Added(line) if line == "line2modified"));
    assert!(matches!(&hunk.lines[3], Line::Added(line) if line == "line3"));
}

#[test]
fn parse_rename_diff_without_changes() {
    let diff = r#"diff --git a/old.txt b/new.txt
similarity index 100%
rename from old.txt
rename to new.txt
"#;

    let files = diff_parser::parse(diff).expect("should parse");
    assert_eq!(files.len(), 1);
    let file = &files[0];
    assert_eq!(file.path, "new.txt");
    assert!(file.hunks.is_empty());
}

#[test]
fn parse_binary_file_diff() {
    let diff = r#"diff --git a/image.png b/image.png
new file mode 100644
index 0000000..e69de29
Binary files /dev/null and b/image.png differ
"#;

    let files = diff_parser::parse(diff).expect("should parse");
    assert_eq!(files.len(), 1);
    let file = &files[0];
    assert_eq!(file.path, "image.png");
    assert!(file.hunks.is_empty());
}

#[test]
fn parse_multiple_hunks() {
    use engine::diff_parser::Line;

    let diff = r#"diff --git a/foo.txt b/foo.txt
--- a/foo.txt
+++ b/foo.txt
@@ -1,2 +1,2 @@
-line1
-line2
+line1mod
+line2
@@ -4,2 +4,3 @@
 line4
-line5
+line5mod
+line6
"#;

    let files = diff_parser::parse(diff).expect("should parse");
    assert_eq!(files.len(), 1);
    let file = &files[0];
    assert_eq!(file.hunks.len(), 2);

    // Verify first hunk
    let h1 = &file.hunks[0];
    assert_eq!(h1.old_start, 1);
    assert_eq!(h1.new_start, 1);
    assert_eq!(h1.lines.len(), 4);
    assert!(matches!(h1.lines[0], Line::Removed(ref l) if l == "line1"));
    assert!(matches!(h1.lines[2], Line::Added(ref l) if l == "line1mod"));

    // Verify second hunk
    let h2 = &file.hunks[1];
    assert_eq!(h2.old_start, 4);
    assert_eq!(h2.new_start, 4);
    assert_eq!(h2.lines.len(), 4);
    assert!(matches!(h2.lines[1], Line::Removed(ref l) if l == "line5"));
    assert!(matches!(h2.lines[2], Line::Added(ref l) if l == "line5mod"));
    assert!(matches!(h2.lines[3], Line::Added(ref l) if l == "line6"));
}
