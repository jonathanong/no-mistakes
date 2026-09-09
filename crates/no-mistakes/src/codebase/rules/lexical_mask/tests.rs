use super::{csharp_code_mask, swift_code_mask};

fn executable(mask: &str) -> String {
    mask.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[test]
fn csharp_format_clause_does_not_start_a_block_comment() {
    let source = "var formatted = $\"{value:/*}\";\nnew Command(async () => { });\n";
    let mask = csharp_code_mask(source);
    assert!(
        executable(&mask).contains("newCommand(async()=>{})"),
        "{mask:?}"
    );
}

#[test]
fn csharp_parenthesized_ternary_colon_is_not_a_format_clause() {
    let source = "var text = $\"{(flag ? new Command(async () => { }) : other)}\";\n";
    let mask = csharp_code_mask(source);
    assert!(
        executable(&mask).contains("newCommand(async()=>{})"),
        "{mask:?}"
    );
}

#[test]
fn swift_regex_literal_does_not_open_a_string() {
    let source = "let quote = #/\"/#\nprint(\"after regex\")\n";
    let mask = swift_code_mask(source);
    assert!(mask.contains("print("), "{mask:?}");
}

#[test]
fn swift_matching_hash_escape_does_not_end_the_string() {
    let source = "let s = #\"abc \\#\"# print(fake) end\"#\nprint(\"after escape\")\n";
    let mask = swift_code_mask(source);
    assert!(mask.contains("print("), "{mask:?}");
    assert!(!executable(&mask).contains("print(fake)"), "{mask:?}");
}

#[test]
fn swift_interpolation_closer_keeps_a_token_boundary() {
    let source = "let named = \"\\(print)\"\n(f)()\n";
    let mask = swift_code_mask(source);
    assert!(mask.contains("print)"), "{mask:?}");
    assert!(!mask.contains("print("), "{mask:?}");
}

#[test]
fn swift_hash_that_is_not_a_literal_stays_code() {
    let source = "#if os(iOS)\nprint(\"real\")\n#endif\n";
    let mask = swift_code_mask(source);
    assert!(mask.contains("print("), "{mask:?}");
}

#[test]
fn swift_escape_at_eof_does_not_panic() {
    let mask = swift_code_mask("let s = \"\\");
    assert!(mask.starts_with("let s = "), "{mask:?}");
}

#[test]
fn swift_hashed_multiline_string_masks_content() {
    let source = "let s = #\"\"\"\nprint(fake)\n\"\"\"#\nprint(\"real\")\n";
    let mask = swift_code_mask(source);
    assert!(mask.contains("print("), "{mask:?}");
    assert!(!executable(&mask).contains("print(fake)"), "{mask:?}");
}

#[test]
fn swift_raw_backslash_without_hash_stays_inside_the_string() {
    let source = "let s = #\"abc \\ def\"#\nprint(\"real\")\n";
    let mask = swift_code_mask(source);
    assert!(mask.contains("print("), "{mask:?}");
}

#[test]
fn csharp_indexer_and_extra_brackets_in_interpolation() {
    let source = "var text = $\"{arr[0]]}\";\nvar extra = $\"{x)}\";\nvar after = new Command(async () => { });\n";
    let mask = csharp_code_mask(source);
    assert!(
        executable(&mask).contains("newCommand(async()=>{})"),
        "{mask:?}"
    );
}

#[test]
fn csharp_unterminated_format_clause_stops_at_eof() {
    let mask = csharp_code_mask("var formatted = $\"{value:/*");
    assert!(mask.contains("var formatted = "), "{mask:?}");
}
