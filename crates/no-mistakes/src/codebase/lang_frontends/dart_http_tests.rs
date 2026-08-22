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

#[test]
fn commented_http_calls_are_skipped() {
    assert!(extract_http_paths(r#"// http.get(Uri.parse("/api/users"));"#).is_empty());
}

#[test]
fn raw_uri_literals_extract() {
    let paths = extract_http_paths(
        r#"
Uri.parse(r'/api/users');
http.get(r"/api/health");
"#,
    );
    assert!(paths.contains(&"/api/users".to_string()));
    assert!(paths.contains(&"/api/health".to_string()));
}

#[test]
fn uri_fragments_are_stripped() {
    assert!(extract_http_paths(r#"Uri.parse("/api/users#details");"#)
        .contains(&"/api/users".to_string()));
}

#[test]
fn interpolated_uris_are_skipped() {
    assert!(extract_http_paths(r#"Uri.parse("/api/$userId");"#).is_empty());
}

#[test]
fn suffix_receivers_are_skipped() {
    assert!(
        extract_http_paths(r#"MyUri.parse("/api/users"); myhttp.get("/api/health");"#).is_empty()
    );
}

#[test]
fn query_slashes_on_hosted_urls_are_not_paths() {
    assert!(extract_http_paths(r#"Uri.parse("https://example.com?next=/api/users");"#).is_empty());
}
