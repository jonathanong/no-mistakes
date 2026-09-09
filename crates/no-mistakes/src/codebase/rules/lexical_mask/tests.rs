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
