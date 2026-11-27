use super::*;

#[test]
fn laravel_literal_routes_still_extract() {
    let source = r#"
Route::get('/api/users', [UserController::class, 'index']);
"#;
    let routes = extract_php_routes(source, true, false);
    assert!(routes.iter().any(|(path, _)| path == "/api/users"));
}

#[test]
fn laravel_resource_expands_collection_and_member_paths() {
    let source = r#"
Route::resource('users', UserController::class);
Route::resource('/posts', \App\Http\Controllers\PostController::class);
"#;
    let routes = extract_php_routes(source, true, false);
    assert!(routes
        .iter()
        .any(|(path, handler)| { path == "/users" && handler.contains("UserController") }));
    assert!(routes
        .iter()
        .any(|(path, handler)| { path == "/users/:user" && handler.contains("UserController") }));
    assert!(routes
        .iter()
        .any(|(path, handler)| { path == "/posts" && handler.contains("PostController") }));
    assert!(routes
        .iter()
        .any(|(path, handler)| { path == "/posts/:post" && handler.contains("PostController") }));
    assert!(routes.contains(&("/users".into(), "UserController".into())));
    assert!(routes
        .iter()
        .all(|(path, handler)| path != "/users/create" && handler != "UserController.index"));
}

#[test]
fn laravel_resource_trailing_comma_still_expands() {
    let routes = extract_php_routes(
        "Route::resource('users', UserController::class,);\n",
        true,
        false,
    );
    assert!(routes.iter().any(|(path, _)| path == "/users"));
    assert!(routes.iter().any(|(path, _)| path == "/users/:user"));
}

#[test]
fn laravel_api_resource_expands_collection_and_member_paths() {
    let source = r#"
Route::apiResource('accounts', UserController::class);
Route::apiResource('/posts', \App\Http\Controllers\PostController::class);
"#;
    let routes = extract_php_routes(source, true, false);
    assert!(routes
        .iter()
        .any(|(path, handler)| { path == "/accounts" && handler.contains("UserController") }));
    assert!(routes.iter().any(|(path, handler)| {
        path == "/accounts/:account" && handler.contains("UserController")
    }));
    assert!(routes
        .iter()
        .any(|(path, handler)| { path == "/posts" && handler.contains("PostController") }));
    assert!(routes
        .iter()
        .any(|(path, handler)| { path == "/posts/:post" && handler.contains("PostController") }));
    assert!(routes.contains(&("/accounts".into(), "UserController".into())));
    assert!(routes.iter().all(|(path, handler)| {
        path != "/accounts/create" && handler != "UserController.index"
    }));
}

#[path = "php_http_resource_tests.rs"]
mod resource_tests;

#[test]
fn symfony_attribute_literal_routes_extract() {
    let source = r#"
#[Route('/health', methods: ['GET'])]
public function health(): void {}

#[Route('/items', methods: ['POST'])]
class ItemController {}
"#;
    let routes = extract_php_routes(source, false, true);
    assert!(routes.contains(&("/health".into(), "health".into())));
    assert!(routes.contains(&("/items".into(), "ItemController".into())));
}

#[test]
fn computed_attribute_path_is_not_a_route() {
    let source = r#"
#[Route($prefix . '/users')]
public function users(): void {}
"#;
    assert!(extract_php_routes(source, false, true).is_empty());
}

#[test]
fn yaml_literal_routes_extract() {
    let source = r#"
users:
  path: /users
  controller: App\Controller\UsersController
"#;
    let routes = extract_yaml_routes(source);
    assert!(routes
        .iter()
        .any(|(path, handler)| { path == "/users" && handler.contains("UsersController") }));
}

#[test]
fn yaml_controller_before_path_extracts() {
    let source = r#"
item:
  controller: App\Controller\ItemController
  path: /items
"#;
    let routes = extract_yaml_routes(source);
    assert!(routes
        .iter()
        .any(|(path, handler)| { path == "/items" && handler == "App.Controller.ItemController" }));
}
