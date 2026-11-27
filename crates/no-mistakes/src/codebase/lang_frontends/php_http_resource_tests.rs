use super::super::*;

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
