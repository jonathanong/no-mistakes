use super::*;

#[test]
fn assertion_start_tracks_comments_quotes_and_nested_calls() {
    let source = r#"expect(
  // misleading expect( and ) delimiters
  /* another expect( and ) pair */
  getPackageJson("escaped \" parenthesis )").devDependencies.foo,
).toBe('1.2.3')"#;
    let match_start = source.find("devDependencies").unwrap();
    let ranges = source_ranges(source).assertions;

    assert_eq!(assertion_start(&ranges, match_start), Some(0));
}

#[test]
fn assertion_start_returns_none_without_an_open_expect() {
    let source = "closed(); packageJson.devDependencies.foo";
    let match_start = source.find("devDependencies").unwrap();
    let ranges = source_ranges(source).assertions;

    assert_eq!(assertion_start(&ranges, match_start), None);
}

#[test]
fn assertion_ranges_keep_an_unclosed_expect_available() {
    let source = "expect(\n  packageJson.devDependencies.foo";
    let match_start = source.find("devDependencies").unwrap();
    let ranges = source_ranges(source).assertions;

    assert_eq!(assertion_start(&ranges, match_start), Some(0));
}

#[test]
fn assertion_start_skips_ended_inner_assertions() {
    let source = r#"expect(
  (() => {
    expect(condition).toBe(true)
    return packageJson.devDependencies.foo
  })(),
).toBe('1.2.3')"#;
    let match_start = source.find("devDependencies").unwrap();
    let ranges = source_ranges(source).assertions;

    assert_eq!(assertion_start(&ranges, match_start), Some(0));
}

#[test]
fn expect_token_detection_requires_a_standalone_name() {
    let start = super::assertion_ranges::expect_token_start;
    assert_eq!(start(b"(", 0, &[]), None);
    assert_eq!(start(b"myexpect(", 8, &[]), None);
    assert_eq!(start(b"myexpect.soft(", 13, &[]), None);
    assert_eq!(start(b"$expect.poll(", 12, &[]), None);
    assert_eq!(start(b"helpers.expect.soft(", 19, &[]), None);
    assert_eq!(start(b"expect (", 7, &[]), Some(0));
    assert_eq!(start(b"expect.soft(", 11, &[]), Some(0));
    assert_eq!(start(b"expect.poll(", 11, &[]), Some(0));
    assert_eq!(start(b"expect<string>(", 14, &[]), Some(0));
    assert_eq!(start(b"expect<Array<string>>(", 21, &[]), Some(0));
    assert_eq!(
        start(b"expect<(value: string) => boolean>(", 34, &[]),
        Some(0)
    );
    assert_eq!(start(b"helper.expect<string>(", 21, &[]), None);
}

#[test]
fn assertion_ranges_skip_comments_before_the_call_parenthesis() {
    for source in [
        "expect /* context */ (packageJson.dependencies.foo).toBe('1.2.3')",
        "expect // context\n(packageJson.dependencies.foo).toBe('1.2.3')",
        "expect<string> /* context */ (packageJson.dependencies.foo).toBe('1.2.3')",
        "expect /* context */ <string>(packageJson.dependencies.foo).toBe('1.2.3')",
        "expect.soft /* context */ (packageJson.dependencies.foo).toBe('1.2.3')",
        "expect.soft /* context */ <Array<string /* > */>> /* call */ (packageJson.dependencies.foo).toBe('1.2.3')",
    ] {
        let match_start = source.find("dependencies").unwrap();
        let ranges = source_ranges(source).assertions;

        assert_eq!(assertion_start(&ranges, match_start), Some(0), "{source}");
    }

    let helper = "helper.expect /* context */ <string>(packageJson.dependencies.foo).toBe('1.2.3')";
    let match_start = helper.find("dependencies").unwrap();
    assert_eq!(
        assertion_start(&source_ranges(helper).assertions, match_start),
        None
    );
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
        let ranges = source_ranges(source).assertions;

        assert_eq!(assertion_start(&ranges, match_start), Some(0), "{modifier}");
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
        let ranges = source_ranges(&source).assertions;

        assert_eq!(
            assertion_start(&ranges, match_start),
            Some(expect_start),
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
        let ranges = source_ranges(&source).assertions;

        assert_eq!(
            assertion_start(&ranges, match_start),
            Some(0),
            "{expression}"
        );
        assert!(ranges.contains(&(0, close)), "{expression}: {ranges:?}");
    }
}

#[test]
fn division_does_not_hide_a_same_line_assertion() {
    for prefix in [
        "const ratio = total / divisor; ",
        "const ratio = count++ / total; ",
        "const ratio = count-- / total; ",
        "const ratio = ++count / total; ",
    ] {
        let source =
            format!("{prefix}expect(\n  packageJson.devDependencies.foo,\n).toBe('1.2.3')");
        let match_start = source.find("devDependencies").unwrap();
        let expect_start = source.find("expect").unwrap();
        let ranges = source_ranges(&source).assertions;

        assert_eq!(
            assertion_start(&ranges, match_start),
            Some(expect_start),
            "{prefix}"
        );
    }
}

#[test]
fn multiline_javascript_strings_keep_later_assertions_visible() {
    for prefix in [
        "const value = 'line\\\ncontinuation';\n",
        "const value = 'line\\\r\ncontinuation';\r\n",
        "const value = `line\ncontinuation`;\n",
    ] {
        let source =
            format!("{prefix}expect(\n  packageJson.devDependencies.foo,\n).toBe('1.2.3')");
        let match_start = source.find("devDependencies").unwrap();
        let expect_start = source.find("expect").unwrap();
        let ranges = source_ranges(&source).assertions;

        assert_eq!(
            assertion_start(&ranges, match_start),
            Some(expect_start),
            "{prefix:?}"
        );
    }
}
