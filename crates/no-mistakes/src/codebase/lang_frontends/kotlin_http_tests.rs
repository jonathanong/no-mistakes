use super::extract_http_routes;

#[test]
fn spring_literals_combine_class_prefix_and_method_path() {
    let routes = extract_http_routes(
        r#"
@RequestMapping("/api")
class Users {
  @GetMapping("/users")
  fun listUsers(): Any = null
  @PostMapping(path = "users")
  @ResponseBody
  fun createUser(): Any = null
}
"#,
    );
    assert!(routes.contains(&("/api/users".into(), "listUsers".into())));
    assert!(routes.contains(&("/api/users".into(), "createUser".into())));
}

#[test]
fn package_private_handlers_extract() {
    let routes = extract_http_routes(
        r#"
class Users {
  @GetMapping("/ok")
  fun ok(): Any = null
}
"#,
    );
    assert!(routes.contains(&("/ok".into(), "ok".into())));
}

#[test]
fn computed_and_empty_mappings_are_skipped() {
    let routes = extract_http_routes(
        r#"
class Computed {
  @GetMapping(PREFIX)
  fun hidden(): Any = null
  @GetMapping
  fun empty(): Any = null
  @GetMapping(path = "/users", produces = "application/json")
  fun extra(): Any = null
}
"#,
    );
    assert!(routes.is_empty());
}
