use super::*;

#[test]
fn flask_and_fastapi_literal_decorators_extract_handlers() {
    let source = r#"
@app.route("/users")
def users():
    return []

@bp.get("/ping")
def ping():
    return "ok"

@router.post("/items")
async def create_item():
    return {}
"#;
    let routes = extract_http_routes(source);
    assert!(routes.contains(&("/users".into(), "users".into())));
    assert!(routes.contains(&("/ping".into(), "ping".into())));
    assert!(routes.contains(&("/items".into(), "create_item".into())));
}

#[test]
fn computed_flask_path_is_not_a_route() {
    let source = r#"
prefix = "/api"
@app.route(prefix + "/users")
def users():
    return []
"#;
    assert!(extract_http_routes(source).is_empty());
}

#[test]
fn django_path_still_extracts() {
    let source = r#"
urlpatterns = [
    path("users/", views.user_list),
]
"#;
    assert_eq!(
        extract_http_routes(source),
        vec![("users/".into(), "views.user_list".into())]
    );
}
