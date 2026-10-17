use super::{has_disable_comment, has_disable_file_comment, has_disable_line_comment};

#[test]
fn disable_line_handles_parser_edge_cases() {
    for (source, line) in [
        ("case 1:// no-mistakes-disable-line my-rule", 1),
        (
            "curl http://example.com;# no-mistakes-disable-line my-rule",
            1,
        ),
        (
            "const value = `hello\n`; blocked(); // no-mistakes-disable-line my-rule",
            2,
        ),
        ("echo /*\necho ok # no-mistakes-disable-line my-rule", 2),
        (
            "if (ok) /a\\/\\/b/.test(x); // no-mistakes-disable-line my-rule",
            1,
        ),
        (
            "const n = 1 + /[//]/.test(v); // no-mistakes-disable-line my-rule",
            1,
        ),
        ("select foo-- no-mistakes-disable-line my-rule", 1),
        ("const x = \"a\"/2; // no-mistakes-disable-line my-rule", 1),
    ] {
        assert!(has_disable_line_comment(source, line, "my-rule"));
    }
}

#[test]
fn bom_prefixed_line_directives_are_detected() {
    for source in [
        "\u{FEFF}# no-mistakes-disable-line my-rule",
        "\u{FEFF}-- no-mistakes-disable-line my-rule",
    ] {
        assert!(has_disable_line_comment(source, 1, "my-rule"));
    }
}

#[test]
fn bom_prefixed_next_line_directives_are_detected() {
    for source in [
        "\u{FEFF}# no-mistakes-disable-next-line my-rule\nblocked()",
        "\u{FEFF}-- no-mistakes-disable-next-line my-rule\nselect 1",
    ] {
        assert!(has_disable_comment(source, 2, "my-rule"));
    }
}

#[test]
fn disable_line_detected_after_parenthesized_division() {
    for source in [
        "(a + b) / 2; // no-mistakes-disable-line my-rule",
        "foo(a) / 2; // no-mistakes-disable-line my-rule",
        ") / 2; // no-mistakes-disable-line my-rule",
    ] {
        assert!(has_disable_line_comment(source, 1, "my-rule"));
    }
}

#[test]
fn disable_line_ignores_directive_inside_midline_block_comment() {
    let source = "const x = 1; /* start\n// no-mistakes-disable-line my-rule\n*/";
    assert!(!has_disable_line_comment(source, 2, "my-rule"));
}

#[test]
fn disable_line_ignores_directive_inside_word_started_block_comment() {
    let source = "return /* start\n// no-mistakes-disable-line my-rule\n*/ value";
    assert!(!has_disable_line_comment(source, 2, "my-rule"));
}

#[test]
fn disable_line_ignores_directive_inside_operator_started_block_comment() {
    let source = "const x = value + /* start\n// no-mistakes-disable-line my-rule\n*/ other";
    assert!(!has_disable_line_comment(source, 2, "my-rule"));
}

#[test]
fn disable_line_ignores_directive_inside_token_adjacent_block_comment() {
    for source in [
        "foo/* start\n// no-mistakes-disable-line my-rule\n*/",
        "switch (x)/* start\n// no-mistakes-disable-line my-rule\n*/",
    ] {
        assert!(!has_disable_line_comment(source, 2, "my-rule"));
    }
}

#[test]
fn disable_line_ignores_dash_prefixed_option() {
    let source = "tool --no-mistakes-disable-line my-rule";
    assert!(!has_disable_line_comment(source, 1, "my-rule"));
}

#[test]
fn file_disable_stops_at_decrement_expression() {
    let source = "--counter;\n// no-mistakes-disable-file my-rule";
    assert!(!has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_allows_hash_attribute_shaped_hash_comment() {
    let source = "#[section]\n# no-mistakes-disable-file my-rule\nset -e";
    assert!(has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_stops_at_rust_inner_attribute() {
    let source = "#![no_std]\n// no-mistakes-disable-file my-rule";
    assert!(!has_disable_file_comment(source, "my-rule"));
}

#[test]
fn file_disable_stops_at_rust_outer_attribute() {
    let source = "#[cfg(test)]\n// no-mistakes-disable-file my-rule";
    assert!(!has_disable_file_comment(source, "my-rule"));
}

#[test]
fn html_comments_match_file_line_and_next_line_directives() {
    let file = "<!-- no-mistakes-disable-file my-rule -->\ncode";
    let line = "PIN <!-- no-mistakes-disable-line my-rule-->";
    let next = "<!-- no-mistakes-disable-next-line my-rule -->\nPIN";
    assert!(has_disable_file_comment(file, "my-rule"));
    assert!(!has_disable_file_comment(file, "other-rule"));
    assert!(has_disable_line_comment(line, 1, "my-rule"));
    assert!(!has_disable_line_comment(line, 1, "other-rule"));
    assert!(has_disable_comment(next, 2, "my-rule"));
    assert!(!has_disable_comment(next, 2, "other-rule"));
    assert_eq!(
        super::matching_disable_directive(file, Some(2), "my-rule"),
        Some(super::DisableDirective::File { line: 1 })
    );
    assert_eq!(
        super::matching_disable_directive(line, Some(1), "my-rule"),
        Some(super::DisableDirective::Line { line: 1 })
    );
    assert_eq!(
        super::matching_disable_directive(next, Some(2), "my-rule"),
        Some(super::DisableDirective::NextLine { line: 1 })
    );
    assert_eq!(super::disabled_statement_lines(line, "my-rule"), vec![1]);
    assert_eq!(super::disabled_statement_lines(next, "my-rule"), vec![2]);
}

#[test]
fn html_comments_do_not_hide_other_comment_syntax() {
    for (source, line, enabled) in [
        (
            "const s = \"<!-- no-mistakes-disable-line my-rule -->\";",
            1,
            false,
        ),
        ("a<!--\n// no-mistakes-disable-line my-rule", 2, true),
        ("if (a<!--b) // no-mistakes-disable-line my-rule", 1, true),
        (
            "<!-- note --> x // no-mistakes-disable-line my-rule",
            1,
            true,
        ),
        ("<!-- no-mistakes-disable-line my-rule", 1, false),
        ("// no-mistakes-disable-line my-rule", 1, true),
        ("# no-mistakes-disable-line my-rule", 1, true),
        ("-- no-mistakes-disable-line my-rule", 1, true),
        ("<!-- no-mistakes-disable-line my-rule-extra-->", 1, false),
    ] {
        assert_eq!(
            has_disable_line_comment(source, line, "my-rule"),
            enabled,
            "{source}"
        );
    }
    assert!(!has_disable_file_comment(
        "code\n<!-- no-mistakes-disable-file my-rule -->",
        "my-rule"
    ));
    assert!(has_disable_file_comment(
        "<!-- copyright -->\n<!-- no-mistakes-disable-file my-rule -->",
        "my-rule"
    ));
    assert!(!has_disable_file_comment(
        "<!-- no-mistakes-disable-file my-rule --> code",
        "my-rule"
    ));
}
