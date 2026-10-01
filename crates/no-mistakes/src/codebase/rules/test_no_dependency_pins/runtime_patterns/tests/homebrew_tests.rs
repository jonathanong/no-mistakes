use super::super::super::{check_source, compile_options, Options};
use super::pins;

#[test]
fn homebrew_formulae_need_homebrew_context() {
    for line in [
        "brew install postgresql@18",
        "brew services start postgresql@18",
        "brew install --cask temurin@21",
        "/opt/homebrew/opt/postgresql@18/bin",
        "/usr/local/Cellar/postgresql@18/18.1",
        "Homebrew formula postgresql@18.",
    ] {
        assert!(
            pins(line)
                .iter()
                .any(|pin| pin.starts_with("postgresql@") || pin.starts_with("temurin@")),
            "{line}"
        );
    }
    for line in [
        "undici@1.0.1",
        "pnpm@12",
        "packageManager: 'pnpm@12.0.0'",
        "brewery@2",
        "brew install postgresql",
        "brew services start postgresql@\\d+",
    ] {
        assert!(pins(line).is_empty(), "{line}");
    }
}

#[test]
fn a_homebrew_formula_version_has_at_most_two_components() {
    // `foo@1.2.3` is not a formula shape; it must not be cut down to `foo@1.2`.
    for line in [
        "brew install foo@1.2.3",
        "brew install postgresql@18.1.2",
        "brew install foo@1.2.3.4",
    ] {
        assert!(pins(line).is_empty(), "{line}: {:?}", pins(line));
    }
    // A dot that ends a sentence still closes the formula.
    for (line, expected) in [
        ("brew install foo@18.", "foo@18"),
        ("brew install openssl@3.5.", "openssl@3.5"),
        ("Run brew install foo@18. Then continue.", "foo@18"),
        ("brew install foo@1.2, brew install bar@3", "foo@1.2"),
    ] {
        assert_eq!(pins(line).first().map(String::as_str), Some(expected));
    }
}

fn readings(line: &str) -> Vec<(String, String)> {
    let options = compile_options(&Options::default()).unwrap();
    check_source("src/app.test.mts", line, &options)
        .into_iter()
        .filter_map(|finding| Some((finding.target?, finding.import?)))
        .collect()
}

#[test]
fn a_homebrew_formula_is_reported_once_not_also_as_an_action_ref() {
    // The action-ref pattern reads `core/postgresql@18` and `opt/postgresql@18`
    // as `owner/repo@ref`; the Homebrew pattern owns that text.
    let formula = ("versioned Homebrew formula", "postgresql@18");
    for line in [
        "brew install homebrew/core/postgresql@18",
        "brew install homebrew/core/postgresql@18.",
        "/opt/homebrew/opt/postgresql@18/bin",
        "/usr/local/Cellar/postgresql@18/18.1",
    ] {
        let expected = vec![(formula.0.to_string(), formula.1.to_string())];
        assert_eq!(readings(line), expected, "{line}");
    }
}

#[test]
fn an_action_ref_on_a_homebrew_line_is_still_reported() {
    // Dropping action refs on Homebrew lines would hide this pinned action.
    let sha = "de0fac2e4500dabe0009e67214ff5f5447ce83dd";
    let line = format!("uses: Homebrew/actions/setup-homebrew@{sha}");
    assert_eq!(
        readings(&line),
        [(
            "exact action ref".to_string(),
            format!("actions/setup-homebrew@{sha}")
        )]
    );
    // A formula and an unrelated action ref on one line are both reported.
    let both = readings("brew install foo@18 && uses: actions/checkout@v4");
    assert_eq!(both.len(), 2, "{both:?}");
}

#[test]
fn a_uses_value_that_is_not_an_action_ref_keeps_its_formula() {
    // A formula name may hold `+`, an action name may not, so no action-ref
    // finding covers this text and dropping the formula would report nothing.
    for line in [
        "uses: Homebrew/core/libc++@18",
        "- uses: homebrew/core/libc++@18",
        "{ name: x, uses: Homebrew/core/libc++@18 }",
        "expect(workflow).toContain('uses: Homebrew/core/libc++@18')",
        "brew install homebrew/core/libc++@18",
    ] {
        assert_eq!(readings(line), [formula("libc++@18")], "{line}");
    }
    // A neighbouring action ref is still its own finding.
    let mut both = readings("uses: Homebrew/actions/setup-homebrew@4 && brew install libc++@18");
    both.sort();
    assert_eq!(
        both,
        [action_ref("actions/setup-homebrew@4"), formula("libc++@18")]
    );
}

fn action_ref(pin: &str) -> (String, String) {
    ("exact action ref".to_string(), pin.to_string())
}

fn formula(pin: &str) -> (String, String) {
    ("versioned Homebrew formula".to_string(), pin.to_string())
}

#[test]
fn a_uses_value_on_a_homebrew_line_is_an_action_ref_not_a_formula() {
    // `setup-homebrew@4` and `brew@4.1` have the formula shape and the line says
    // Homebrew, but the text is the value of `uses:`, so it is only an action ref.
    for (line, expected) in [
        (
            "uses: Homebrew/actions/setup-homebrew@4",
            "actions/setup-homebrew@4",
        ),
        (
            "uses: Homebrew/actions/setup-homebrew@4.1",
            "actions/setup-homebrew@4.1",
        ),
        ("uses: Homebrew/brew@4.1", "Homebrew/brew@4.1"),
        ("uses:   Homebrew/brew@4", "Homebrew/brew@4"),
        ("uses : Homebrew/brew@4", "Homebrew/brew@4"),
        (
            "- uses: Homebrew/actions/setup-homebrew@4",
            "actions/setup-homebrew@4",
        ),
        (
            "uses: 'Homebrew/actions/setup-homebrew@4'",
            "actions/setup-homebrew@4",
        ),
        (
            "uses: \"Homebrew/actions/setup-homebrew@4\"",
            "actions/setup-homebrew@4",
        ),
        (
            "\"uses\": \"Homebrew/actions/setup-homebrew@4\"",
            "actions/setup-homebrew@4",
        ),
        (
            "{ name: x, uses: Homebrew/actions/setup-homebrew@4 }",
            "actions/setup-homebrew@4",
        ),
        (
            "expect(workflow).toContain('uses: Homebrew/actions/setup-homebrew@4')",
            "actions/setup-homebrew@4",
        ),
        // A key starts at an opening quote (any kind), indentation inside a string,
        // a flow-mapping opener, or a list marker.
        (
            "const workflow = 'uses: Homebrew/actions/setup-homebrew@4'",
            "actions/setup-homebrew@4",
        ),
        (
            "const workflow = \"uses: Homebrew/actions/setup-homebrew@4\"",
            "actions/setup-homebrew@4",
        ),
        (
            "const workflow = `uses: Homebrew/actions/setup-homebrew@4`",
            "actions/setup-homebrew@4",
        ),
        (
            "expect(workflow).toContain('  uses: Homebrew/actions/setup-homebrew@4')",
            "actions/setup-homebrew@4",
        ),
        (
            "{\"uses\": \"Homebrew/actions/setup-homebrew@4\"}",
            "actions/setup-homebrew@4",
        ),
        (
            "with: {uses: Homebrew/actions/setup-homebrew@4}",
            "actions/setup-homebrew@4",
        ),
        (
            "steps: [uses: Homebrew/actions/setup-homebrew@4]",
            "actions/setup-homebrew@4",
        ),
        // A comma starts a key inside a flow mapping or sequence, after any
        // number of entries (spaces and nested flow values included).
        (
            "{ name: Setup Homebrew, uses: Homebrew/actions/setup-homebrew@4 }",
            "actions/setup-homebrew@4",
        ),
        (
            "steps: [name: x, with: y, uses: Homebrew/actions/setup-homebrew@4]",
            "actions/setup-homebrew@4",
        ),
        (
            "{ env: { HOMEBREW: yes }, uses: Homebrew/actions/setup-homebrew@4 }",
            "actions/setup-homebrew@4",
        ),
        (
            "steps: [{ name: x }, [a, b], uses: Homebrew/actions/setup-homebrew@4]",
            "actions/setup-homebrew@4",
        ),
        // A quoted scalar before the key may hold a closer or a comma.
        (
            "{ name: \"}\", uses: Homebrew/actions/setup-homebrew@4 }",
            "actions/setup-homebrew@4",
        ),
        (
            "{ name: '}', uses: Homebrew/actions/setup-homebrew@4 }",
            "actions/setup-homebrew@4",
        ),
        (
            "{ \"name\": \"]\", uses: Homebrew/actions/setup-homebrew@4 }",
            "actions/setup-homebrew@4",
        ),
        (
            "\"{ name: \\\"}\\\", uses: Homebrew/actions/setup-homebrew@4 }\"",
            "actions/setup-homebrew@4",
        ),
        (
            "{ name: don't, uses: Homebrew/actions/setup-homebrew@4 }",
            "actions/setup-homebrew@4",
        ),
        (
            "toEqual(['{ name: x, uses: Homebrew/actions/setup-homebrew@4 }'])",
            "actions/setup-homebrew@4",
        ),
        // A closer left over from a collection opened on an earlier line does not
        // cancel the opener that follows it.
        (
            "], { name: x, uses: Homebrew/actions/setup-homebrew@4 }",
            "actions/setup-homebrew@4",
        ),
        // The key may follow a JavaScript escape with no separator before it.
        (
            "'name: Setup\\nuses: Homebrew/actions/setup-homebrew@4'",
            "actions/setup-homebrew@4",
        ),
        (
            "'steps:\\n  - uses: \\'Homebrew/actions/setup-homebrew@4\\''",
            "actions/setup-homebrew@4",
        ),
        (
            "uses: Homebrew/actions/setup-homebrew@4 # v4",
            "actions/setup-homebrew@4 # v4",
        ),
    ] {
        assert_eq!(readings(line), [action_ref(expected)], "{line}");
    }
}

#[test]
fn a_formula_is_still_a_formula_when_uses_is_not_its_key() {
    // Only the text between the `uses:` key and the pin decides: a later formula,
    // a different key that ends in `uses`, and `uses` without a colon do not hide it.
    for line in [
        "brew install homebrew/core/postgresql@18",
        "brew reuses: homebrew/core/postgresql@18",
        "brew package.uses: homebrew/core/postgresql@18",
        "brew steps/uses: homebrew/core/postgresql@18",
        "brew pre-uses: homebrew/core/postgresql@18",
        // Prose, not a key: the word before `uses:` is not a key opener.
        "Homebrew uses: homebrew/core/postgresql@18",
        "expect(output).toContain('Homebrew uses: homebrew/core/postgresql@18')",
        "brew note: uses: homebrew/core/postgresql@18",
        "brew install foo - uses: homebrew/core/postgresql@18",
        // A comma is a key start only inside a flow mapping or sequence.
        "Homebrew, uses: homebrew/core/postgresql@18",
        "expect(output).toContain('Homebrew, uses: homebrew/core/postgresql@18')",
        "Homebrew {see}, uses: homebrew/core/postgresql@18",
        "Homebrew [see note], uses: homebrew/core/postgresql@18",
        "Homebrew {a: [b]}, uses: homebrew/core/postgresql@18",
        "Homebrew { a: \"[\" }, uses: homebrew/core/postgresql@18",
        "-uses: homebrew/core/postgresql@18",
        "uses homebrew/core/postgresql@18",
        "brew $uses: homebrew/core/postgresql@18",
        "brew @uses: homebrew/core/postgresql@18",
        "brew key:uses: homebrew/core/postgresql@18",
        "brew \\uses: homebrew/core/postgresql@18",
        "brew éuses: homebrew/core/postgresql@18",
        "brew uses homebrew/core/postgresql@18",
        "brew install postgresql@18 # uses: homebrew/core",
    ] {
        assert_eq!(readings(line), [formula("postgresql@18")], "{line}");
    }
    // A slashless value is not an action ref (`owner/repo@ref`), so no action-ref
    // finding replaces the formula and it must not be dropped.
    for line in [
        "Homebrew uses: postgresql@18",
        "expect(output).toContain('Homebrew uses: postgresql@18')",
        "brew uses: \"postgresql@18\"",
        "\"uses\": \"postgresql@18\" // brew",
    ] {
        assert_eq!(readings(line), [formula("postgresql@18")], "{line}");
    }
    let mut mixed =
        readings("uses: actions/checkout@v4 && brew install homebrew/core/postgresql@18");
    mixed.sort();
    assert_eq!(
        mixed,
        [action_ref("actions/checkout@v4"), formula("postgresql@18")]
    );
    let mut after = readings("uses: Homebrew/actions/setup-homebrew@4 && brew install foo@18");
    after.sort();
    assert_eq!(
        after,
        [action_ref("actions/setup-homebrew@4"), formula("foo@18")]
    );
}
