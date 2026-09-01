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
    assert_eq!(expect_token_start(b"(", 0), None);
    assert_eq!(expect_token_start(b"myexpect(", 8), None);
    assert_eq!(expect_token_start(b"myexpect.soft(", 13), None);
    assert_eq!(expect_token_start(b"$expect.poll(", 12), None);
    assert_eq!(expect_token_start(b"helpers.expect.soft(", 19), None);
    assert_eq!(expect_token_start(b"expect (", 7), Some(0));
    assert_eq!(expect_token_start(b"expect.soft(", 11), Some(0));
    assert_eq!(expect_token_start(b"expect.poll(", 11), Some(0));
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
