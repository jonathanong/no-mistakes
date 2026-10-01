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
        "g(text)",
        "CREATE FUNCTION g(x text DEFAULT 'not body') RETURNS int LANGUAGE sql AS $$ SELECT 2 $$",
    );
    assert_eq!(quoted.body.as_deref(), Some(" SELECT 2 "));

    let sql_string = function_from_definition(
        "s()",
        "CREATE FUNCTION s() RETURNS int LANGUAGE sql AS 'SELECT ''widget'''",
    );
    assert_eq!(sql_string.body.as_deref(), Some("SELECT 'widget'"));

    let fake_event = function_from_definition(
        "e(text)",
        "CREATE FUNCTION e(note text DEFAULT 'returns event_trigger') RETURNS void LANGUAGE plpgsql AS $$ BEGIN RETURN; END $$",
    );
    assert!(!fake_event.returns_event_trigger);
    let commented_event = function_from_definition(
        "c()",
        "CREATE FUNCTION c() /* returns event_trigger */ RETURNS void LANGUAGE plpgsql AS $$ BEGIN RETURN; END $$",
    );
    assert!(!commented_event.returns_event_trigger);
    let event = function_from_definition(
        "ev()",
        "CREATE FUNCTION ev() RETURNS event_trigger LANGUAGE plpgsql AS $$ BEGIN RETURN; END $$",
    );
    assert!(event.returns_event_trigger);

    let escaped = function_from_definition(
        "e()",
        "CREATE FUNCTION e() RETURNS int LANGUAGE sql AS E'SELECT \\'widget\\''",
    );
    assert_eq!(escaped.body.as_deref(), Some("SELECT 'widget'"));

    let atomic_language = function_from_definition(
        "a()",
        "CREATE FUNCTION a() RETURNS int BEGIN ATOMIC SELECT language FROM settings; SELECT 1; END LANGUAGE sql",
    );
    assert_eq!(atomic_language.language.as_deref(), Some("sql"));
    assert!(atomic_language.body.unwrap().contains("SELECT 1"));

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

    let nested = function_from_definition(
        "c()",
        "CREATE FUNCTION c() RETURNS int LANGUAGE sql BEGIN ATOMIC SELECT CASE WHEN 1 THEN 1 END; SELECT 2; END",
    );
    assert!(nested.body.unwrap().contains("SELECT 2"));

    let quoted_language = function_from_definition(
        "q()",
        "CREATE FUNCTION q() RETURNS int LANGUAGE \"sql\" AS $$ SELECT 1 $$",
    );
    assert_eq!(quoted_language.language.as_deref(), Some("sql"));
    assert_eq!(quoted_language.body.as_deref(), Some(" SELECT 1 "));

    let dollar = function_from_definition(
        "d(text)",
        "CREATE FUNCTION d(x text DEFAULT $d$ AS $$fake$$ $d$) RETURNS int LANGUAGE sql AS $$ SELECT 9 $$",
    );
    assert_eq!(dollar.body.as_deref(), Some(" SELECT 9 "));

    let nested_comment = function_from_definition(
        "n()",
        "CREATE FUNCTION n() RETURNS int LANGUAGE sql /* outer /* inner */ AS $$fake$$ */ AS $$ SELECT 7 $$",
    );
    assert_eq!(nested_comment.body.as_deref(), Some(" SELECT 7 "));
}
