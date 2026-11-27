use super::{
    client_matches_route, extract_dotnet, extract_elixir, extract_go, extract_jvm, extract_php,
    extract_python, extract_ruby, extract_rust,
};

#[test]
fn static_literals_keep_a_path_and_computed_urls_do_not() {
    assert_eq!(extract_python(r#"requests.get("/users")"#), vec!["/users"]);
    assert_eq!(
        extract_python(r#"requests.get("https://example.test/users?x=1")"#),
        vec!["/users"]
    );
    assert!(extract_python(r#"requests.get(f"/users/{user_id}")"#).is_empty());
    assert!(extract_python(r#"requests.get(url)"#).is_empty());
    assert_eq!(extract_go(r#"http.Get("/users")"#), vec!["/users"]);
    assert_eq!(extract_go("`unused`\nhttp.Get(`/health`)"), vec!["/health"]);
    assert!(extract_go(r#"http.Get(fmt.Sprintf("/users/%s", id))"#).is_empty());
    assert_eq!(extract_rust(r#"reqwest::get("/users")"#), vec!["/users"]);
    assert!(extract_rust(r#"reqwest::get(format!("/users/{}", id))"#).is_empty());
    assert_eq!(extract_ruby(r#"Faraday.get("/users")"#), vec!["/users"]);
    assert!(extract_ruby(r#"Faraday.get(path)"#).is_empty());
    assert_eq!(extract_php(r#"Http::get('/users')"#), vec!["/users"]);
    assert!(extract_php(r#"Http::get($url)"#).is_empty());
    assert_eq!(
        extract_jvm(r#"rest.getForObject("/api/users", String.class)"#),
        vec!["/api/users"]
    );
    assert!(extract_jvm(r#"rest.getForObject(url, String.class)"#).is_empty());
    assert!(extract_jvm(r#"map.put("/users", 1)"#).is_empty());
    assert_eq!(extract_elixir(r#"Req.get("/users")"#), vec!["/users"]);
    assert!(extract_elixir(r#"Req.get("/users/" <> id)"#).is_empty());
    assert_eq!(
        extract_dotnet(r#"client.GetAsync("/users")"#),
        vec!["/users"]
    );
    assert!(extract_dotnet(r#"client.GetAsync($"/users/{id}")"#).is_empty());
}

#[test]
fn client_paths_match_normalized_route_patterns() {
    assert!(client_matches_route("/users", "/users"));
    assert!(client_matches_route("/users", "users/"));
    assert!(client_matches_route("/api/users", "/api/users"));
    assert!(!client_matches_route("/users", "/accounts"));
}
