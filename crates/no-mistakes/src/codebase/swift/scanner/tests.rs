use super::*;

#[test]
fn scanner_handles_invalid_labels_comments_and_unterminated_input() {
    assert_eq!(
        find_label_colon("rename: \"wrong\" name \"missing\"", "name"),
        None
    );
    assert_eq!(
        find_label_colon("// name: \"commented\"\nrealName: 1", "name"),
        None
    );
    assert_eq!(
        find_matching_delimiter("call(unterminated", 4, '(', ')'),
        None
    );
    assert_eq!(read_quoted_string(r#""unterminated"#, 0), None);

    let mut scanner = Scanner::new("/* skipped */ name: \"ok\"");
    let index = scanner.next_code_index().expect("code after block comment");
    assert_eq!(
        "/* skipped */ name: \"ok\""[index..].trim_start(),
        "name: \"ok\""
    );
}

#[test]
fn scanner_reads_escaped_strings() {
    let (value, next) = read_quoted_string(r#""hello \"swift\"" trailing"#, 0)
        .expect("escaped string should parse");
    assert_eq!(value, r#"hello \"swift\""#);
    assert_eq!(&r#""hello \"swift\"" trailing"#[next..], " trailing");
}
