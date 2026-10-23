use super::{expand_brace_members, extract_elixir_imports};

#[test]
fn exact_alias_import_and_use_extract() {
    let source = r#"
alias MyApp.User
alias MyApp.{Role, Team}
import MyApp.Accounts
use Phoenix.Controller
"#;
    assert_eq!(
        extract_elixir_imports(source),
        vec![
            "MyApp.Accounts",
            "MyApp.Role",
            "MyApp.Team",
            "MyApp.User",
            "Phoenix.Controller"
        ]
    );
}

#[test]
fn nested_brace_and_wildcard_aliases_are_non_edges() {
    let source = r#"
alias MyApp.{Foo.{Bar, Baz}}
alias MyApp.*
alias MyApp*
import MyApp.{User, *}
alias MyApp.{User
alias MyApp.{User}}
"#;
    assert!(extract_elixir_imports(source).is_empty());
}

#[test]
fn trailing_comment_after_brace_alias_still_expands() {
    let source = "alias MyApp.{User} # keep\n";
    assert_eq!(extract_elixir_imports(source), vec!["MyApp.User"]);
}

#[test]
fn brace_alias_without_trailing_newline_expands() {
    assert_eq!(
        extract_elixir_imports("alias MyApp.{User}"),
        vec!["MyApp.User"]
    );
    assert!(extract_elixir_imports("alias MyApp.{User}}").is_empty());
}

#[test]
fn expand_brace_members_covers_skip_and_keep_arms() {
    assert!(expand_brace_members("MyApp", "User").is_empty());
    assert!(expand_brace_members("MyApp", "{User").is_empty());
    assert!(expand_brace_members("MyApp", "{Foo.{Bar}}").is_empty());
    assert!(expand_brace_members("MyApp", "{User, *}").is_empty());
    assert_eq!(
        expand_brace_members("MyApp", "{User}"),
        vec!["MyApp.User".to_string()]
    );
    assert_eq!(
        expand_brace_members("MyApp", "{User}\n"),
        vec!["MyApp.User".to_string()]
    );
    assert_eq!(
        expand_brace_members("MyApp", "{User} # x"),
        vec!["MyApp.User".to_string()]
    );
}

#[test]
fn next_line_tuple_does_not_become_a_brace_alias() {
    let source = "alias MyApp.User\n{:ok, x} = foo()\nalias MyApp.Role\n{Foo, Bar} = y\n";
    assert_eq!(
        extract_elixir_imports(source),
        vec!["MyApp.Role", "MyApp.User"]
    );
}

#[test]
fn space_brace_import_and_dotted_members_expand() {
    let source = r#"
alias MyApp {User, Role}
import MyApp.{Accounts}
use MyApp.{Repo.Reader}
alias MyApp.{skip_me, Team}
"#;
    assert_eq!(
        extract_elixir_imports(source),
        vec![
            "MyApp.Accounts",
            "MyApp.Repo.Reader",
            "MyApp.Role",
            "MyApp.Team",
            "MyApp.User",
        ]
    );
}
