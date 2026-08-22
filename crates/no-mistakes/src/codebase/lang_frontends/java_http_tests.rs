use super::extract_http_routes;

#[test]
fn spring_literals_combine_class_prefix_and_method_path() {
    let routes = extract_http_routes(
        r#"
@RequestMapping("/api")
public class Users {
  @GetMapping("/users")
  public Object listUsers() { return null; }
  @PostMapping(path = "users")
  @ResponseBody
  public Object createUser() { return null; }
}
"#,
    );
    assert!(routes.contains(&("/api/users".into(), "listUsers".into())));
    assert!(routes.contains(&("/api/users".into(), "createUser".into())));
}

#[test]
fn computed_and_empty_mappings_are_skipped() {
    let routes = extract_http_routes(
        r#"
public class Computed {
  @GetMapping(PREFIX)
  public Object hidden() { return null; }
  @GetMapping
  public Object empty() { return null; }
}
"#,
    );
    assert!(routes.is_empty());
}
