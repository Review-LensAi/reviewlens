use engine::diff_parser;

#[test]
fn parse_empty_diff_returns_no_files() {
    let files = diff_parser::parse("").expect("should parse");
    assert!(files.is_empty());
}

#[test]
#[should_panic]
fn parse_non_empty_diff_is_unimplemented() {
    let diff = "diff --git a/a b/b";
    let _ = diff_parser::parse(diff).unwrap();
}
