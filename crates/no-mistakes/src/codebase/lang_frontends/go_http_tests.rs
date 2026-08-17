use super::*;

#[test]
fn literal_net_http_and_mux_registrations_extract_handlers() {
    let source = r#"
http.HandleFunc("/health", Health)
mux.Handle("/ready", Ready)
r.Get("/users", Users)
g.POST("/items", CreateItem)
e.PUT("/ping", Ping)
app.Delete("/status", Status)
"#;
    let routes = extract_http_routes(source);
    assert!(routes.contains(&("/health".into(), "Health".into())));
    assert!(routes.contains(&("/ready".into(), "Ready".into())));
    assert!(routes.contains(&("/users".into(), "Users".into())));
    assert!(routes.contains(&("/items".into(), "CreateItem".into())));
    assert!(routes.contains(&("/ping".into(), "Ping".into())));
    assert!(routes.contains(&("/status".into(), "Status".into())));
}

#[test]
fn computed_http_pattern_is_not_a_route() {
    let source = r#"
func Register(pattern string) {
    http.Handle(pattern, Health)
}
"#;
    assert!(extract_http_routes(source).is_empty());
}

#[test]
fn asynq_task_handle_func_is_not_a_route() {
    let source = r#"
mux.HandleFunc("mail:welcome", HandleWelcome)
"#;
    assert!(extract_http_routes(source).is_empty());
}

#[test]
fn comment_route_examples_are_not_routes() {
    let source = crate::codebase::lang_frontends::strip::strip_comments_keep_strings(
        r#"
// http.HandleFunc("/docs-example", Hidden)
http.HandleFunc("/health", Health)
"#,
    );
    assert_eq!(
        extract_http_routes(&source),
        vec![("/health".into(), "Health".into())]
    );
}
