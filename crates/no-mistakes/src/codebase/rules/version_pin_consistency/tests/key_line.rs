use super::super::key::key_line;

#[test]
fn key_line_covers_comments_headers_quotes_and_missing_segments() {
    assert_eq!(key_line("name = 1\n", "name"), 1);
    assert_eq!(key_line("# name = 1\nname = 2\n", "name"), 2);
    assert_eq!(key_line("// name = 1\nname: 2\n", "name"), 2);
    assert_eq!(key_line("note # name = 1\nname = 2\n", "name"), 2);
    assert_eq!(key_line("note // name = 1\nname = 2\n", "name"), 2);
    assert_eq!(key_line("\"name\" = 1\n", "name"), 1);
    assert_eq!(key_line("'name': 1\n", "name"), 1);
    assert_eq!(key_line("[tools]\naqua = \"1\"\n", "tools"), 1);
    assert_eq!(key_line("[tools.aqua]\nversion = \"1\"\n", "aqua"), 1);
    assert_eq!(key_line("other = 1\n", "missing"), 1);
    assert_eq!(key_line("tools.foo = 1\n", "missing.foo"), 1);
    assert_eq!(
        key_line("tools:\n  aqua:\n    lychee = \"1\"\n", "tools.aqua.lychee"),
        3
    );
    assert_eq!(key_line("renamed = 1\n", "tools.missing.rest"), 1);
    assert_eq!(key_line("name-value = 1\n", "name"), 1);
}
