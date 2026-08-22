use super::extract_http_routes;

#[test]
fn map_get_and_http_get_literals_extract_handlers() {
    let routes = extract_http_routes(
        r#"
app.MapGet("/users", ListUsers);
app.MapPost("/users", UserHandlers.CreateUser);
[HttpGet("orders/{id}")]
public Task<IActionResult> GetOrder(int id) => Task.FromResult<IActionResult>(null);
[Microsoft.AspNetCore.Mvc.HttpGet("/prefixed")]
public IActionResult Prefixed() => Ok();
[HttpPost("/orders")]
public IActionResult CreateOrder() => Ok();
"#,
    );
    assert_eq!(
        routes,
        vec![
            ("/orders".into(), "CreateOrder".into()),
            ("/orders/{id}".into(), "GetOrder".into()),
            ("/prefixed".into(), "Prefixed".into()),
            ("/users".into(), "ListUsers".into()),
            ("/users".into(), "UserHandlers.CreateUser".into()),
        ]
    );
}

#[test]
fn computed_and_empty_attribute_routes_are_skipped() {
    let routes = extract_http_routes(
        r#"
app.MapGet(path, ListUsers);
app.MapGet("/users", () => "ok");
[HttpGet]
public IActionResult Index() => Ok();
[HttpGet(Name = "named")]
public IActionResult Named() => Ok();
"#,
    );
    assert!(routes.is_empty());
}

#[test]
fn comment_map_get_examples_are_not_routes() {
    let source = super::super::csharp_strip::strip_comments_keep_strings(
        r#"
// app.MapGet("/docs", Hidden);
app.MapGet("/users", ListUsers);
var unused = "not a \"route\"";
"#,
    );
    assert_eq!(
        extract_http_routes(&source),
        vec![("/users".into(), "ListUsers".into())]
    );
}
