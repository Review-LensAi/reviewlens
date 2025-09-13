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
