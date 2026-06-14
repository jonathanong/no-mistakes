use super::*;

#[test]
fn assignment_index_skips_quoted_equals_and_arrow_types() {
    let source = r#"const ROUTES: Record<"=", () => { slug: string }> = {}"#;

    assert_eq!(
        assignment_index(source, "const ROUTES".len()),
        Some(source.rfind('=').unwrap() + 1)
    );
}

#[test]
fn assignment_index_returns_none_without_initializer() {
    assert_eq!(assignment_index("const ROUTES: Type", 0), None);
}

#[test]
fn assignment_index_handles_escaped_quotes() {
    let source = r#"const ROUTES = "not \" = done"; const NEXT = {}"#;

    assert_eq!(
        assignment_index(source, "const ROUTES".len()),
        Some("const ROUTES =".len())
    );
    assert_eq!(
        assignment_index(source, source.find("const NEXT").unwrap()),
        Some(source.rfind('=').unwrap() + 1)
    );
}

#[test]
fn assignment_index_handles_escaped_backslashes_in_strings() {
    let source = r#"const ROUTES: Record<"not \\ done", string> = {}"#;

    assert_eq!(
        assignment_index(source, "const ROUTES".len()),
        Some(source.rfind('=').unwrap() + 1)
    );
}

#[test]
fn matching_brace_ignores_comments_and_regex_literals() {
    let source = r#"{
  // }
  block: /* } */ true,
  pattern: /[{}\/]/,
  nested: { value: "}" },
}"#;

    assert_eq!(matching_brace(source, 0), Some(source.len() - 1));
}

#[test]
fn matching_brace_handles_unclosed_values() {
    assert_eq!(matching_brace("{ value: /unterminated", 0), None);
}

#[test]
fn matching_brace_ignores_quoted_escaped_braces() {
    let source = r#"{ value: `\` } still quoted`, next: '\' } still quoted' }"#;

    assert_eq!(matching_brace(source, 0), Some(source.len() - 1));
}

#[test]
fn top_level_value_end_ignores_nested_and_regex_commas() {
    assert_eq!(top_level_value_end(r#"/[,{}]/, next: true"#), 7);
    assert_eq!(top_level_value_end(r#"{ value: "," }, next: true"#), 14);
}

#[test]
fn top_level_value_end_handles_comments_and_division() {
    assert_eq!(top_level_value_end("value // comment, ignored\n, next"), 16);
    assert_eq!(top_level_value_end("// comment\n, next"), 11);
    assert_eq!(top_level_value_end("/* comment */, next"), 13);
    assert_eq!(top_level_value_end("a / b, next"), 5);
}

#[test]
fn top_level_value_end_skips_escaped_regex_character_classes() {
    assert_eq!(top_level_value_end(r#"/[\]\/,]/, next: true"#), 9);
}
