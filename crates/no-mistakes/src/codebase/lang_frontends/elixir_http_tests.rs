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

#[test]
fn literal_scope_prefix_joins_nested_verbs() {
    let routes = extract_http_routes(
        r#"
    get "/users", MyAppWeb.UserController, :index
    scope "/api" do
      get "/items", MyAppWeb.UserController, :index
      scope "/v1" do
        post "/orders", MyAppWeb.UserController, :create
      end
    end
    get "/health", MyAppWeb.UserController, :index
"#,
    );
    assert!(routes.contains(&("/users".into(), "MyAppWeb.UserController.index".into())));
    assert!(routes.contains(&("/api/items".into(), "MyAppWeb.UserController.index".into())));
    assert!(routes.contains(&(
        "/api/v1/orders".into(),
        "MyAppWeb.UserController.create".into()
    )));
    assert!(routes.contains(&("/health".into(), "MyAppWeb.UserController.index".into())));
}

#[test]
fn computed_scope_skips_inner_verbs() {
    let routes = extract_http_routes(
        r#"
    scope path do
      get "/hidden", UserController, :index
    end
    get "/visible", UserController, :index
"#,
    );
    assert!(routes.contains(&("/visible".into(), "UserController.index".into())));
    assert!(routes.iter().all(|(path, _)| path != "/hidden"));
}
