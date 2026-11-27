use super::{
    extract_dotnet, extract_elixir, extract_go, extract_jvm, extract_php, extract_ruby,
    extract_rust, normalize_route_pattern,
};

fn python_paths(source: &str) -> Vec<String> {
    super::extract_python(source, &super::super::strip::mask_strings(source))
}

#[test]
fn static_literals_keep_a_path_and_computed_urls_do_not() {
    assert_eq!(python_paths(r#"requests.get("/users")"#), vec!["/users"]);
    assert!(python_paths(r#"requests.get("https://api.example.invalid/users?x=1")"#).is_empty());
    assert!(python_paths(r#"requests.get("//api.example.invalid/users")"#).is_empty());
    assert_eq!(
        python_paths(r#"requests.get("/users?next=https://api.example.invalid")"#),
        vec!["/users"]
    );
    assert!(python_paths(r#"requests.get(f"/users/{user_id}")"#).is_empty());
    assert!(python_paths(r#"requests.get(url)"#).is_empty());
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
        python_paths(r#"requests.get("http://localhost:8080/users?x=1")"#),
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
    assert!(python_paths(r#"requests.get("http://localhost.example.invalid/users")"#).is_empty());
    assert!(extract_go(r#"http.Get("http://192.0.2.1/users")"#).is_empty());
    assert!(extract_go(r#"http.Get("ftp://localhost/users")"#).is_empty());
}

#[test]
fn python_lint_ignores_strings_and_retains_encoded_literals() {
    let code = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/lang-frontends/http-clients/python/computed.py"
    ));
    assert!(python_paths(code).is_empty());
    let code = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/lang-frontends/http-clients/python/client.py"
    ));
    assert!(python_paths(code).contains(&"/customers/jane%20doe".to_string()));
    assert!(python_paths(r#"requests.get("/users/%s")"#).is_empty());
}

#[test]
fn jvm_uri_creation_requires_a_supported_client_call() {
    assert!(extract_jvm(r#"URI.create("/api/users")"#).is_empty());
    assert_eq!(
        extract_jvm(r#"rest.getForObject(URI.create("/api/users"), String.class)"#),
        vec!["/api/users"]
    );
}

#[test]
fn client_paths_match_normalized_route_patterns() {
    assert_eq!(normalize_route_pattern("users/"), "/users");
    assert_eq!(normalize_route_pattern("/api/users?query=1"), "/api/users");
    assert_eq!(normalize_route_pattern(""), "/");
    assert_eq!(normalize_route_pattern("/users/{id:int}"), "/users/:id");
}
