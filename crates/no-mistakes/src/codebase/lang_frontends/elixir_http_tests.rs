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
