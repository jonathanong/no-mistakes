use super::{
    extract_dotnet, extract_elixir, extract_go, extract_jvm, extract_php, extract_python,
    extract_ruby, extract_rust, normalize_route_pattern,
};

#[test]
fn static_literals_keep_a_path_and_computed_urls_do_not() {
    assert_eq!(extract_python(r#"requests.get("/users")"#), vec!["/users"]);
    assert!(extract_python(r#"requests.get("https://api.example.invalid/users?x=1")"#).is_empty());
    assert!(extract_python(r#"requests.get("//api.example.invalid/users")"#).is_empty());
    assert_eq!(
        extract_python(r#"requests.get("/users?next=https://api.example.invalid")"#),
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
fn loopback_clients_match_without_treating_other_hosts_as_local() {
    assert_eq!(
        extract_python(r#"requests.get("http://localhost:8080/users?x=1")"#),
        vec!["/users"]
    );
    assert_eq!(
        extract_go(r#"http.Get("http://127.0.0.1/users")"#),
        vec!["/users"]
    );
    assert_eq!(
        extract_rust(r#"reqwest::get("https://[::1]/users")"#),
        vec!["/users"]
    );
    assert!(extract_python(r#"requests.get("http://localhost.example.invalid/users")"#).is_empty());
    assert!(extract_go(r#"http.Get("http://192.0.2.1/users")"#).is_empty());
    assert!(extract_go(r#"http.Get("ftp://localhost/users")"#).is_empty());
}

#[test]
fn client_paths_match_normalized_route_patterns() {
    assert_eq!(normalize_route_pattern("users/"), "/users");
    assert_eq!(normalize_route_pattern("/api/users?query=1"), "/api/users");
    assert_eq!(normalize_route_pattern(""), "/");
}
