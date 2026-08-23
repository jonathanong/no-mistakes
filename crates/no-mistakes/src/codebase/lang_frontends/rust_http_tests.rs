use super::*;
use crate::codebase::lang_frontends::strip::strip_comments_keep_strings;

#[test]
fn axum_actix_and_rocket_literals_extract_handlers() {
    let source = r#"
.route("/users", get(list_users))
.route("/items", post(crate::items::create_item))
web::resource("/ready").route(web::get().to(ready))
#[get("/health")]
pub async fn health() {}
#[post("/ping")]
fn ping() {}
"#;
    let routes = extract_http_routes(source);
    assert!(routes.contains(&("/users".into(), "list_users".into())));
    assert!(routes.contains(&("/items".into(), "create_item".into())));
    assert!(routes.contains(&("/ready".into(), "ready".into())));
    assert!(routes.contains(&("/health".into(), "health".into())));
    assert!(routes.contains(&("/ping".into(), "ping".into())));
}

#[test]
fn actix_whitespace_and_repeated_routes_extract() {
    let source = r#"
web::resource("/ready")
    .route(web::get().to(ready))
    .route(web::post().to(create));
#[actix_web::get("/health")]
async fn health() {}
#[options("/ping")]
fn ping() {}
"#;
    let routes = extract_http_routes(source);
    assert!(routes.contains(&("/ready".into(), "ready".into())));
    assert!(routes.contains(&("/ready".into(), "create".into())));
    assert!(routes.contains(&("/health".into(), "health".into())));
    assert!(routes.contains(&("/ping".into(), "ping".into())));
}

#[test]
fn computed_and_chained_axum_routes_are_non_edges() {
    let source = r#"
.route(path, get(list_users))
.route("/x", get(a).post(b))
web::resource(prefix).route(web::get().to(ready))
#[get(prefix)]
pub async fn hidden() {}
"#;
    assert!(extract_http_routes(source).is_empty());
}

#[test]
fn comment_route_examples_are_not_routes() {
    let source = strip_comments_keep_strings(
        r#"
// .route("/docs-example", get(hidden))
.route("/users", get(list_users))
"#,
    );
    assert_eq!(
        extract_http_routes(&source),
        vec![("/users".into(), "list_users".into())]
    );
}
