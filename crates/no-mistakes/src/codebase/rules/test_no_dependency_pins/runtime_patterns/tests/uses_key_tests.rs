//! Where `uses` is a key on a Homebrew line decides only the reason a pin is
//! reported under (`exact action ref` or `versioned Homebrew formula`), never
//! whether it is reported, so these tests compare reasons and not text.

use super::super::super::{check_source, compile_options, Options};

const ACTION_REF: &str = "exact action ref";
const FORMULA: &str = "versioned Homebrew formula";

/// The line and reason of every finding in `content`.
fn reasons(content: &str) -> Vec<(usize, String)> {
    let options = compile_options(&Options::default()).unwrap();
    check_source("src/app.test.mts", content, &options)
        .into_iter()
        .filter_map(|finding| Some((finding.line, finding.target?)))
        .collect()
}

fn reason(line: usize, reason: &str) -> (usize, String) {
    (line, reason.to_string())
}

#[test]
fn uses_stays_a_key_through_escaped_quotes_at_every_javascript_layer() {
    for line in [
        // A `\\\"` inside a `\"` scalar is a quote of the scalar, so its brace is quoted.
        r#""{ name: \"a\\\"}x\", uses: Homebrew/actions/setup-homebrew@4 }""#,
        r#""{ name: \"a\\\"]x\", uses: Homebrew/actions/setup-homebrew@4 }""#,
        // Two layers write the quote itself with three backslashes.
        r#""{ name: \\\"}\\\", uses: Homebrew/actions/setup-homebrew@4 }""#,
        // One layer with a quoted brace, an escaped quote with no brace, and an
        // escaped backslash before the closing quote.
        r#""{ name: \"}\", uses: Homebrew/actions/setup-homebrew@4 }""#,
        r#""{ name: \"a\\\"b\", uses: Homebrew/actions/setup-homebrew@4 }""#,
        r#""{ name: \"x\\\\\", uses: Homebrew/actions/setup-homebrew@4 }""#,
        // A single-quoted scalar has no backslash escape: the quote ends it, so
        // `a\` is a scalar and the mapping goes on.
        r#"'{ name: \'}\', uses: Homebrew/actions/setup-homebrew@4 }'"#,
        r#"'{ name: \\\'}\\\', uses: Homebrew/actions/setup-homebrew@4 }'"#,
        r#"'{ name: \'a\\\', uses: Homebrew/actions/setup-homebrew@4 }'"#,
        r#"'{ name: \'a\\\', env: \'}\', uses: Homebrew/actions/setup-homebrew@4 }'"#,
        // Raw YAML with an escaped quote in a plain-quoted scalar.
        r#"{ name: "a\"}x", uses: Homebrew/actions/setup-homebrew@4 }"#,
    ] {
        assert_eq!(reasons(line), [reason(1, ACTION_REF)], "{line}");
    }
}

#[test]
fn a_comma_after_a_closed_escaped_mapping_is_prose_not_a_uses_key() {
    for line in [
        // The scalar ends at its closing quote, so the brace after it closes the mapping.
        r#"brew { a: \"x\\\"{\" }, uses: homebrew/core/postgresql@18"#,
        r#"brew { a: \\\"x\\\" }, uses: homebrew/core/postgresql@18"#,
        // In single quotes a backslash is literal, so the quote after it ends the
        // scalar and the brace after that closes the mapping.
        r#"{ a: \'x\\\' }, note: \'y\', uses: homebrew/core/postgresql@18"#,
        r#"'{ name: \'a\\\'}x\', uses: Homebrew/actions/setup-homebrew@4 }'"#,
        // An even run of backslashes writes no quote, so the brace it hides counts.
        r#"brew { a: \\"}, uses: homebrew/core/postgresql@18"#,
        r#"brew { a: \\\\"}, uses: homebrew/core/postgresql@18"#,
    ] {
        assert_eq!(reasons(line), [reason(1, FORMULA)], "{line}");
    }
}

#[test]
fn a_flow_collection_opened_on_an_earlier_line_is_not_known() {
    // Limit: each line is read on its own, so a comma on a continuation line is
    // not inside a collection the line never opened. The action ref is still
    // reported once, under the formula reason. A `uses` that starts the line is
    // a key wherever the collection was opened.
    for content in [
        "steps: [{ name: x,\n  env: { A: 1 }, uses: Homebrew/actions/setup-homebrew@4 }]",
        "steps: [{ name: x,\n  id: a, uses: Homebrew/actions/setup-homebrew@4 }]",
        "- { name: x,\n    id: a, uses: Homebrew/actions/setup-homebrew@4,\n    with: { y: 1 } }",
    ] {
        assert_eq!(reasons(content), [reason(2, FORMULA)], "{content}");
    }
    let starts_the_line = "steps: [{ name: x,\n  uses: Homebrew/actions/setup-homebrew@4 }]";
    assert_eq!(reasons(starts_the_line), [reason(2, ACTION_REF)]);
}

#[test]
fn a_block_scalar_and_a_value_on_the_next_line_are_not_known() {
    // Limit: a line that starts with `uses:` is a key even inside `run: |`, and a
    // value on the line after `uses:` has no key before it.
    assert_eq!(
        reasons("run: |\n  uses: homebrew/core/postgresql@18"),
        [reason(2, ACTION_REF)]
    );
    assert_eq!(
        reasons("- uses:\n    Homebrew/actions/setup-homebrew@4"),
        [reason(2, FORMULA)]
    );
}

#[test]
fn a_doubled_quote_in_an_escaped_single_quoted_scalar_is_not_known() {
    // Limit: `\'\'` is a quote of the scalar, so `\'it\'\'s}\'` holds `it's}` and
    // `uses` is a key. The scalar is read as ending at the first `\'`, so the
    // brace closes the mapping and the formula reason is chosen.
    let line = r#"{ a: \'it\'\'s}\', uses: Homebrew/actions/setup-homebrew@4 }"#;
    assert_eq!(reasons(line), [reason(1, FORMULA)], "{line}");
}

#[test]
fn a_plain_quoted_scalar_is_read_as_raw_yaml() {
    // Limit: a backslash in a plain-quoted scalar escapes the next byte, as in
    // YAML. When the text is really inside a JavaScript string, `\\"` is an
    // escaped quote and the scalar goes on, but the line alone cannot say so.
    let line = r#"'{ name: "a\\"}x", uses: Homebrew/actions/setup-homebrew@4 }'"#;
    assert_eq!(reasons(line), [reason(1, FORMULA)], "{line}");
}
