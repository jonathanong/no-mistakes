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

#[test]
fn static_resource_forms_emit_one_fact_per_remaining_action() {
    let source = r#"
Route::resource('hidden', UserController::class, ['only' => ['index']]);
Route::resource('articles', UserController::class)->only(['show']);
Route::resource('labels', UserController::class)->names(['index' => 'labels.index'])->only(['index']);
Route::resource('photos.comments', PhotoCommentController::class);
Route::resource(name: 'named-users', controller: UserController::class, only: ['index', 'show']);
Route::apiResource(controller: UserController::class, name: 'named-accounts', except: ['destroy']);
Route::apiResources(['bulk' => UserController::class]);
Route::apiResource('filtered', UserController::class)->only(['index']);
Route::apiResource('excluded', UserController::class)->except(['index']);
Route::resource('notes', UserController::class)->except(['index', 'create', 'edit']);
Route::resource("$prefix/users", UserController::class);
Route::resource($name, UserController::class);
Route::resource(__DIR__ . '/files', UserController::class);
"#;
    let expected = vec![
        ("/hidden", "UserController.index"),
        ("/articles/:article", "UserController.show"),
        ("/labels", "UserController.index"),
        ("/photos/:photo/comments", "PhotoCommentController.index"),
        (
            "/photos/:photo/comments/create",
            "PhotoCommentController.create",
        ),
        ("/photos/:photo/comments", "PhotoCommentController.store"),
        (
            "/photos/:photo/comments/:comment",
            "PhotoCommentController.show",
        ),
        (
            "/photos/:photo/comments/:comment/edit",
            "PhotoCommentController.edit",
        ),
        (
            "/photos/:photo/comments/:comment",
            "PhotoCommentController.update",
        ),
        (
            "/photos/:photo/comments/:comment",
            "PhotoCommentController.destroy",
        ),
        ("/named-users", "UserController.index"),
        ("/named-users/:named-user", "UserController.show"),
        ("/named-accounts", "UserController.index"),
        ("/named-accounts", "UserController.store"),
        ("/named-accounts/:named-account", "UserController.show"),
        ("/named-accounts/:named-account", "UserController.update"),
        ("/bulk", "UserController.index"),
        ("/bulk", "UserController.store"),
        ("/bulk/:bulk", "UserController.show"),
        ("/bulk/:bulk", "UserController.update"),
        ("/bulk/:bulk", "UserController.destroy"),
        ("/filtered", "UserController.index"),
        ("/excluded", "UserController.store"),
        ("/excluded/:excluded", "UserController.show"),
        ("/excluded/:excluded", "UserController.update"),
        ("/excluded/:excluded", "UserController.destroy"),
        ("/notes", "UserController.store"),
        ("/notes/:note", "UserController.show"),
        ("/notes/:note", "UserController.update"),
        ("/notes/:note", "UserController.destroy"),
    ];
    assert_eq!(route_pairs(source), owned_pairs(expected));
}

#[test]
fn quoted_multiline_and_slash_prefix_resources_expand() {
    let source = r#"
Route::resource(
    "widgets",
    UserController::class,
    ["only" => ["index"]]
);
Route::resource('admin/photos', UserController::class)->only(['show']);
Route::resource('widgets', \App\Http\Controllers\UserController::class)->only(['store']);
"#;
    let expected = vec![
        ("/widgets", "UserController.index"),
        ("/widgets", "App.Http.Controllers.UserController.store"),
        ("/admin/photos/:photo", "UserController.show"),
    ];
    assert_eq!(route_pairs(source), owned_pairs(expected));
}

#[test]
fn dynamic_or_ambiguous_resource_forms_stay_non_edges() {
    let source = r#"
Route::resource('clash', UserController::class, ['only' => ['index']])->only(['show']);
Route::resource('both', UserController::class, ['only' => ['index'], 'except' => ['show']]);
Route::apiResource('drafts', UserController::class)->only(['create']);
Route::resource('blank', UserController::class)->only([]);
Route::resource("$prefix/users", UserController::class);
Route::resource($name, UserController::class);
Route::resource(__DIR__ . '/files', UserController::class);
"#;
    assert!(extract_php_routes(source, true, false).is_empty());
}

fn route_pairs(source: &str) -> Vec<(String, String)> {
    let mut pairs = extract_php_routes(source, true, false);
    pairs.sort();
    pairs
}

fn owned_pairs(pairs: Vec<(&str, &str)>) -> Vec<(String, String)> {
    let mut pairs = pairs
        .into_iter()
        .map(|(path, handler)| (path.to_string(), handler.to_string()))
        .collect::<Vec<_>>();
    pairs.sort();
    pairs
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
