use super::extract_http_routes;

#[test]
fn phoenix_verb_literals_extract() {
    let routes = extract_http_routes(
        r#"
    get "/users", MyAppWeb.UserController, :index
    post "/users", MyAppWeb.UserController, :create
"#,
    );
    assert!(routes.contains(&("/users".into(), "MyAppWeb.UserController.index".into())));
    assert!(routes.contains(&("/users".into(), "MyAppWeb.UserController.create".into())));
}

#[test]
fn paths_without_leading_slash_are_normalized() {
    let routes = extract_http_routes(r#"    get "users", MyAppWeb.UserController, :index"#);
    assert!(routes.contains(&("/users".into(), "MyAppWeb.UserController.index".into())));
}

#[test]
fn moduledoc_heredoc_examples_are_not_routes() {
    let routes = extract_http_routes(
        r#"
@moduledoc """
  get "/users", MyAppWeb.UserController, :index
"""
"#,
    );
    assert!(routes.is_empty());
}

#[test]
fn resources_and_computed_paths_are_skipped() {
    let routes = extract_http_routes(
        r#"
    resources "/users", UserController
    get path, UserController, :index
    get "/users", UserController
"#,
    );
    assert!(routes.is_empty());
}
