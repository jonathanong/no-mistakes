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
