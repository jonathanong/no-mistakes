use super::extract_http_paths;

#[test]
fn uri_parse_and_http_verbs_extract_absolute_paths() {
    let paths = extract_http_paths(
        r#"
final users = Uri.parse("/api/users");
http.get(Uri.parse("/api/users"));
http.post("/api/users");
http.get("https://example.com/api/health");
"#,
    );
    assert!(paths.contains(&"/api/users".to_string()));
    assert!(paths.contains(&"/api/health".to_string()));
}

#[test]
fn computed_uris_are_skipped() {
    assert!(extract_http_paths(r#"http.get(Uri.parse(prefix + "/users"));"#).is_empty());
}
