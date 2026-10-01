use super::super::function::function_from_definition;

#[test]
fn language_and_body_ignore_headers_and_keep_atomic_sql() {
    let parameter = function_from_definition(
        "f(language text)",
        "CREATE FUNCTION f(language text) RETURNS int LANGUAGE sql AS $$ SELECT 1 $$",
    );
    assert_eq!(parameter.language.as_deref(), Some("sql"));
    assert_eq!(parameter.body.as_deref(), Some(" SELECT 1 "));

    let quoted = function_from_definition(
        "g()",
        "CREATE FUNCTION g() RETURNS int LANGUAGE sql AS 'not body' AS $$ SELECT 2 $$",
    );
    assert_eq!(quoted.body.as_deref(), Some(" SELECT 2 "));

    let commented = function_from_definition(
        "h()",
        "CREATE FUNCTION h() RETURNS int LANGUAGE sql AS $$ SELECT 3 $$ -- as $$nope$$",
    );
    assert_eq!(commented.body.as_deref(), Some(" SELECT 3 "));

    let atomic = function_from_definition(
        "i()",
        "CREATE FUNCTION i() RETURNS int LANGUAGE sql BEGIN ATOMIC SELECT 4; END",
    );
    assert_eq!(atomic.language.as_deref(), Some("sql"));
    assert_eq!(atomic.body.as_deref(), Some(" SELECT 4; "));
}
