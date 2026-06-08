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
    let src = "// comment\n/* placeholder */\n";
    assert_eq!(classify_content(src, "rs"), ContentKind::CommentsOnly);
}

#[test]
fn test_css_has_content() {
    let src = "/* reset */\nbody { margin: 0; }\n";
    assert_eq!(classify_content(src, "css"), ContentKind::HasContent);
}

#[test]
fn test_md_html_comment_only() {
    let src = "<!-- comment -->\n";
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

#[test]
fn test_c_style_blank_line_between_comments() {
    // Empty line between // comments hits the early continue (line 47).
    let src = "// first\n\n// second\n";
    assert_eq!(classify_content(src, "ts"), ContentKind::CommentsOnly);
}

#[test]
fn test_c_style_code_after_block_close() {
    // Code following */ on the same line while in_block triggers line 55.
    let src = "/* start\n*/ const x = 1;\n";
    assert_eq!(classify_content(src, "ts"), ContentKind::HasContent);
}

#[test]
fn test_sql_blank_line_between_dash_comments() {
    // Empty line in SQL content hits the early continue (line 95).
    let src = "-- first\n\n-- second\n";
    assert_eq!(classify_content(src, "sql"), ContentKind::CommentsOnly);
}

#[test]
fn test_sql_multiline_block_comment_only() {
    // Multi-line /* */ block exercises the in_block path (lines 97-105).
    let src = "/*\n still in block\n*/\n";
    assert_eq!(classify_content(src, "sql"), ContentKind::CommentsOnly);
}

#[test]
fn test_sql_code_after_multiline_block_close() {
    // Code after */ when in_block=true reaches return true (lines 101-102).
    let src = "/* comment\n*/ SELECT 1;\n";
    assert_eq!(classify_content(src, "sql"), ContentKind::HasContent);
}

#[test]
fn test_sql_inline_block_comment_with_code() {
    // Single-line /* ... */ with trailing code reaches return true (lines 112-114).
    let src = "/* comment */ SELECT 1;\n";
    assert_eq!(classify_content(src, "sql"), ContentKind::HasContent);
}

#[test]
fn test_md_blank_line_in_comment_context() {
    // Empty line in markdown content hits the early continue (line 140).
    let src = "<!-- first -->\n\n<!-- second -->\n";
    assert_eq!(classify_content(src, "md"), ContentKind::CommentsOnly);
}

#[test]
fn test_md_content_after_close_in_multiline_comment() {
    // Content after --> while in_comment=true reaches return true (line 147).
    let src = "<!--\n still commenting --> real text\n";
    assert_eq!(classify_content(src, "md"), ContentKind::HasContent);
}

#[test]
fn test_md_content_on_same_line_after_comment_close() {
    // Content after --> on same line as <!-- reaches return true (line 156).
    let src = "<!-- comment --> some text\n";
    assert_eq!(classify_content(src, "md"), ContentKind::HasContent);
}

#[test]
fn test_c_style_chained_block_comments_only() {
    // /* a */ /* b */ — two chained block comments, no real code.
    let src = "/* a */ /* b */\n";
    assert_eq!(classify_content(src, "ts"), ContentKind::CommentsOnly);
}

#[test]
fn test_c_style_chained_block_then_code() {
    // /* a */ /* b */ code — chained comments then real code.
    let src = "/* a */ /* b */ const x = 1;\n";
    assert_eq!(classify_content(src, "ts"), ContentKind::HasContent);
}

#[test]
fn test_c_style_chained_block_after_multiline_close() {
    // Multi-line block followed by another block on the closing line.
    let src = "/*\n still in block */ /* another block */\n";
    assert_eq!(classify_content(src, "ts"), ContentKind::CommentsOnly);
}

#[test]
fn test_c_style_tail_unclosed_block() {
    // /* a */ /* unclosed — unclosed tail returns false (no content).
    let src = "/* a */ /* unclosed\n";
    assert_eq!(classify_content(src, "ts"), ContentKind::CommentsOnly);
}

#[test]
fn test_c_style_tail_line_comment_after_block() {
    // /* a */ // line comment — tail starts with //, treated as comment.
    let src = "/* a */ // line comment\n";
    assert_eq!(classify_content(src, "ts"), ContentKind::CommentsOnly);
}

#[test]
fn test_sql_chained_block_comments_only() {
    // /* a */ /* b */ in SQL — chained blocks, no real code.
    let src = "/* a */ /* b */\n";
    assert_eq!(classify_content(src, "sql"), ContentKind::CommentsOnly);
}

#[test]
fn test_sql_chained_block_then_code() {
    // /* a */ /* b */ SELECT 1 — chained blocks then real SQL.
    let src = "/* a */ /* b */ SELECT 1;\n";
    assert_eq!(classify_content(src, "sql"), ContentKind::HasContent);
}

#[test]
fn test_sql_tail_dash_comment_after_block() {
    // /* a */ -- dash comment — tail starts with --, treated as comment.
    let src = "/* a */ -- comment\n";
    assert_eq!(classify_content(src, "sql"), ContentKind::CommentsOnly);
}

#[test]
fn test_md_chained_html_comments_only() {
    // <!-- a --> <!-- b --> — two chained HTML comments, no real content.
    let src = "<!-- a --> <!-- b -->\n";
    assert_eq!(classify_content(src, "md"), ContentKind::CommentsOnly);
}

#[test]
fn test_md_chained_html_then_content() {
    // <!-- a --> <!-- b --> text — chained comments then real content.
    let src = "<!-- a --> <!-- b --> real text\n";
    assert_eq!(classify_content(src, "md"), ContentKind::HasContent);
}

#[test]
fn test_md_tail_unclosed_comment() {
    // <!-- a --> <!-- unclosed — unclosed tail returns false.
    let src = "<!-- a --> <!-- unclosed\n";
    assert_eq!(classify_content(src, "md"), ContentKind::CommentsOnly);
}
