use super::*;

#[test]
fn assertion_start_tracks_comments_quotes_and_nested_calls() {
    let source = r#"expect(
  // misleading expect( and ) delimiters
  /* another expect( and ) pair */
  getPackageJson("escaped \" parenthesis )").devDependencies.foo,
).toBe('1.2.3')"#;
    let match_start = source.find("devDependencies").unwrap();

    assert_eq!(assertion_start(source, match_start), 0);
}

#[test]
fn assertion_start_falls_back_without_an_open_expect() {
    let source = "closed(); packageJson.devDependencies.foo";
    let match_start = source.find("devDependencies").unwrap();

    assert_eq!(assertion_start(source, match_start), match_start);
}

#[test]
fn expect_token_detection_requires_a_standalone_name() {
    assert_eq!(expect_token_start(b"(", 0), None);
    assert_eq!(expect_token_start(b"myexpect(", 8), None);
    assert_eq!(expect_token_start(b"expect (", 7), Some(0));
}
