use super::*;

#[test]
fn test_empty_string() {
    assert_eq!(classify_content("", "ts"), ContentKind::Empty);
}

#[test]
fn test_whitespace_only() {
    assert_eq!(classify_content("   \n\t\n  ", "js"), ContentKind::Empty);
}

#[test]
fn test_ts_line_comment_only() {
    let src = "// this is a comment\n// another comment\n";
    assert_eq!(classify_content(src, "ts"), ContentKind::CommentsOnly);
}

#[test]
fn test_ts_block_comment_only() {
    let src = "/* block\n   comment\n*/\n";
    assert_eq!(classify_content(src, "ts"), ContentKind::CommentsOnly);
}

#[test]
fn test_ts_has_content() {
    let src = "// comment\nconst x = 1;\n";
    assert_eq!(classify_content(src, "ts"), ContentKind::HasContent);
}

#[test]
fn test_tsx_has_content() {
    let src = "export default function App() { return <div />; }\n";
    assert_eq!(classify_content(src, "tsx"), ContentKind::HasContent);
}

#[test]
fn test_mjs_comments_only() {
    let src = "// @ts-nocheck\n/* nothing here */\n";
    assert_eq!(classify_content(src, "mjs"), ContentKind::CommentsOnly);
}

#[test]
fn test_inline_block_comment_then_code() {
    let src = "/* comment */ const x = 1;\n";
    assert_eq!(classify_content(src, "js"), ContentKind::HasContent);
}

#[test]
fn test_sql_line_comment_only() {
    let src = "-- drop table\n-- dangerous\n";
    assert_eq!(classify_content(src, "sql"), ContentKind::CommentsOnly);
}

#[test]
fn test_sql_has_content() {
    let src = "-- comment\nSELECT 1;\n";
    assert_eq!(classify_content(src, "sql"), ContentKind::HasContent);
}

#[test]
fn test_rs_comments_only() {
    let src = "// TODO: implement\n/* placeholder */\n";
    assert_eq!(classify_content(src, "rs"), ContentKind::CommentsOnly);
}

#[test]
fn test_css_has_content() {
    let src = "/* reset */\nbody { margin: 0; }\n";
    assert_eq!(classify_content(src, "css"), ContentKind::HasContent);
}

#[test]
fn test_md_html_comment_only() {
    let src = "<!-- TODO: write docs -->\n";
    assert_eq!(classify_content(src, "md"), ContentKind::CommentsOnly);
}

#[test]
fn test_md_multiline_comment_only() {
    let src = "<!--\n  multi-line\n  comment\n-->\n";
    assert_eq!(classify_content(src, "md"), ContentKind::CommentsOnly);
}

#[test]
fn test_md_has_content() {
    let src = "<!-- comment -->\n# Heading\n";
    assert_eq!(classify_content(src, "md"), ContentKind::HasContent);
}

#[test]
fn test_unknown_ext_always_has_content() {
    let src = "// this looks like a comment but extension is unknown\n";
    assert_eq!(classify_content(src, "xyz"), ContentKind::HasContent);
}

#[test]
fn test_unknown_ext_whitespace_is_empty() {
    assert_eq!(classify_content("   \n", "xyz"), ContentKind::Empty);
}
