use super::*;

#[test]
fn assertion_start_tracks_comments_quotes_and_nested_calls() {
    let source = r#"expect(
  // misleading expect( and ) delimiters
  /* another expect( and ) pair */
  getPackageJson("escaped \" parenthesis )").devDependencies.foo,
).toBe('1.2.3')"#;
    let match_start = source.find("devDependencies").unwrap();
    let ranges = assertion_ranges(source);

    assert_eq!(assertion_start(&ranges, match_start), 0);
}

#[test]
fn assertion_start_falls_back_without_an_open_expect() {
    let source = "closed(); packageJson.devDependencies.foo";
    let match_start = source.find("devDependencies").unwrap();
    let ranges = assertion_ranges(source);

    assert_eq!(assertion_start(&ranges, match_start), match_start);
}

#[test]
fn assertion_ranges_keep_an_unclosed_expect_available() {
    let source = "expect(\n  packageJson.devDependencies.foo";
    let match_start = source.find("devDependencies").unwrap();
    let ranges = assertion_ranges(source);

    assert_eq!(assertion_start(&ranges, match_start), 0);
}

#[test]
fn expect_token_detection_requires_a_standalone_name() {
    let start = super::assertion_ranges::expect_token_start;
    assert_eq!(start(b"(", 0), None);
    assert_eq!(start(b"myexpect(", 8), None);
    assert_eq!(start(b"myexpect.soft(", 13), None);
    assert_eq!(start(b"$expect.poll(", 12), None);
    assert_eq!(start(b"helpers.expect.soft(", 19), None);
    assert_eq!(start(b"expect (", 7), Some(0));
    assert_eq!(start(b"expect.soft(", 11), Some(0));
    assert_eq!(start(b"expect.poll(", 11), Some(0));
}

#[test]
fn assertion_ranges_include_vitest_modifier_calls() {
    for (modifier, source) in [
        (
            "soft",
            "expect.soft(\n  packageJson.devDependencies.foo,\n).toBe('1.2.3')",
        ),
        (
            "poll",
            "expect.poll(\n  () => packageJson.devDependencies.foo,\n).toBe('1.2.3')",
        ),
    ] {
        let match_start = source.find("devDependencies").unwrap();
        let ranges = assertion_ranges(source);

        assert_eq!(assertion_start(&ranges, match_start), 0, "{modifier}");
    }
}

#[test]
fn assertion_ranges_skip_javascript_regex_literals() {
    for prefix in [
        "const quote = /\"/;\n",
        "const quote = /[\"/]/;\n",
        "const quote = /\\\"/;\n",
        "const quote = /\"\n",
        "return /\"/;\n",
    ] {
        let source =
            format!("{prefix}expect(\n  packageJson.devDependencies.foo,\n).toBe('1.2.3')");
        let match_start = source.find("devDependencies").unwrap();
        let expect_start = source.find("expect").unwrap();
        let ranges = assertion_ranges(&source);

        assert_eq!(
            assertion_start(&ranges, match_start),
            expect_start,
            "{prefix:?}"
        );
    }
}

#[test]
fn regex_literals_inside_expect_do_not_change_parenthesis_depth() {
    for expression in [
        r#"/[\")]/.test(value)"#,
        r#"/[\/)]/.test(value)"#,
        r#"left / right && /[)]/.test(value)"#,
    ] {
        let source =
            format!("expect(\n  {expression} && packageJson.devDependencies.foo,\n).toBe('1.2.3')");
        let match_start = source.find("devDependencies").unwrap();
        let close = source.find(").toBe").unwrap();
        let ranges = assertion_ranges(&source);

        assert_eq!(assertion_start(&ranges, match_start), 0, "{expression}");
        assert!(ranges.contains(&(0, close)), "{expression}: {ranges:?}");
    }
}

#[test]
fn division_does_not_hide_a_same_line_assertion() {
    let source = "const ratio = total / divisor; expect(\n  packageJson.devDependencies.foo,\n).toBe('1.2.3')";
    let match_start = source.find("devDependencies").unwrap();
    let expect_start = source.find("expect").unwrap();
    let ranges = assertion_ranges(source);

    assert_eq!(assertion_start(&ranges, match_start), expect_start);
}
