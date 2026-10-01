use super::inside_flow;

#[test]
fn an_unclosed_opener_is_inside_a_flow_collection() {
    for text in [
        "{",
        "[",
        "{ name: x",
        "{ env: { A: 1 }",
        "steps: [{ name: x }, [a, b]",
        "with: {a: [b]",
        // A closer left over from an earlier line does not cancel what follows.
        "], { name: x",
        "expect(workflow).toContain('{ name: x",
    ] {
        assert!(inside_flow(text), "{text}");
    }
}

#[test]
fn prose_and_closed_collections_are_not_inside_one() {
    for text in [
        "",
        "Homebrew",
        "Homebrew {see}",
        "Homebrew [see note]",
        "Homebrew {a: [b]}",
        "}",
        "]]",
    ] {
        assert!(!inside_flow(text), "{text}");
    }
}

#[test]
fn a_quoted_delimiter_does_not_change_the_depth() {
    // The quoted closer must not end the collection that is still open.
    for text in [
        "{ name: \"}\"",
        "{ name: '}'",
        "{ name: `}`",
        "{ \"name\": \"}\"",
        "[\"]\"",
        "{ a: \"x\\\"}\"",
        "{ a: 'it''s }'",
        // The byte after a backslash or a doubled quote is skipped, and no more.
        "{ a: \"x\\y\", b: \"}\"",
        "{ a: 'it''', b: '}'",
        "{ a: \"x\", b: \"}\"",
        "{ a: \"x, y\", b: '}'",
        // Escaped quotes, as in JSON written inside a JavaScript string.
        "\"{ name: \\\"}\\\"",
        "{ a: \\\"x\\\", b: \\\"}\\\"",
        "{ a: \\'}\\'",
        "{ a: \\`}\\`",
        "{ a: \\\"x\\y}\\\"",
    ] {
        assert!(inside_flow(text), "{text}");
    }
    // A quoted opener must not start a collection that was never opened.
    for text in [
        "{ a: \"{\" }",
        "{ a: '[' }",
        "{ a: `{` }",
        "{ a: \\\"{\\\" }",
        "{ a: 'it''s {' }",
        "{ a: \"x\\\"{\" }",
        "{ \"{\": 1 }",
        // Only single quotes double up to a literal quote.
        "{ a: \"x\"\"}\"",
    ] {
        assert!(!inside_flow(text), "{text}");
    }
}

#[test]
fn a_quote_opens_a_string_only_where_a_scalar_starts() {
    // `don't` and `it's` are plain scalars: the quotes must not pair up and hide
    // the closer between them.
    assert!(!inside_flow("{ a: don't }, b: it's"));
    assert!(!inside_flow("{ a: x\"y }, b: z\"w"));
    // An escaped quote that is not at a scalar start is not an opener either.
    assert!(!inside_flow("{ a: x\\\"y }, b: z\\\"w"));
    // A string that ends is a string, even outside a flow collection.
    assert!(!inside_flow("note: \"{\""));
    assert!(!inside_flow("note: '['"));
    // A JavaScript `\n` escape is not an escaped quote, so it opens nothing and
    // the closer between two of them still counts.
    assert!(!inside_flow("{ a: \\n }, b: \\n"));
    // A quote glued to a word or a hyphen is prose, so the brace inside it counts.
    assert!(inside_flow("{ a: x-\"{\" }"));
}

#[test]
fn a_quote_after_a_comma_opens_a_string() {
    // Entries of a flow sequence: the second quoted scalar follows a comma.
    for text in ["[\"x\", \"}\"", "['x', '}'", "{ a: 1, '}': 2", "[a, \"}\""] {
        assert!(inside_flow(text), "{text}");
    }
}

#[test]
fn a_quote_with_no_end_is_not_a_string() {
    // The outer quote opens a string that never ends, so the text is read again
    // with that quote as a plain character and the brace still counts.
    for text in [
        "foo: '{ name: \"}\"",
        "foo: \"{ name: \\\"}\\\"",
        "toEqual(['{ name: x",
        "toEqual(['{ name: \"}\"",
        "{ a: 'x, b: \"}\"",
        "{ a: \\\"x, b: \"}\"",
    ] {
        assert!(inside_flow(text), "{text}");
    }
    assert!(!inside_flow("{ a: 'x, b: }"));
    assert!(!inside_flow("{ a: \\\"x, b: }"));
}
