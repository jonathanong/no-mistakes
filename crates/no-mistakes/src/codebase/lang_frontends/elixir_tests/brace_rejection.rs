use super::extract_elixir_imports;
use std::path::PathBuf;

fn phoenix_fixture_file(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/lang-frontends/phoenix-routes")
        .join(relative)
}

#[test]
fn malformed_nested_brace_alias_cannot_emit_a_valid_sibling_import() {
    let source = std::fs::read_to_string(phoenix_fixture_file("lib/my_app/app.ex")).unwrap();
    assert_eq!(
        extract_elixir_imports(&source),
        vec!["MyApp.Role", "MyApp.User"],
        "the intentionally missing outer brace must not expose MyApp.GhostNested"
    );
}

#[test]
fn brace_group_followed_by_semicolon_keeps_exact_members() {
    let source = std::fs::read_to_string(phoenix_fixture_file("lib/my_app/app.ex")).unwrap();
    let alias = source
        .lines()
        .find(|line| line.trim_start().starts_with("alias MyApp.{User};"))
        .expect("saved semicolon alias");
    assert_eq!(extract_elixir_imports(alias), vec!["MyApp.User"]);
}
