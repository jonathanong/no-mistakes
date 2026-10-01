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
    let escapes = function_from_definition(
        "esc()",
        "CREATE FUNCTION esc() RETURNS text LANGUAGE sql AS E'a\\n\\t\\r\\b\\f\\\\\\x''y'",
    );
    assert_eq!(escapes.body.as_deref(), Some("a\n\t\r\u{8}\u{c}\\x'y"));
    assert_eq!(super::super::function_escape::unescape("\\"), "\\");

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

    let positional = function_from_definition(
        "p(int)",
        "CREATE FUNCTION p(n int DEFAULT $1) RETURNS int LANGUAGE sql AS $$ SELECT 2 $$",
    );
    assert_eq!(positional.body.as_deref(), Some(" SELECT 2 "));

    let escaped_default = function_from_definition(
        "d(text)",
        "CREATE FUNCTION d(note text DEFAULT E'it\\'s AS $$fake$$') RETURNS int LANGUAGE sql AS $$ SELECT 8 $$",
    );
    assert_eq!(escaped_default.body.as_deref(), Some(" SELECT 8 "));
    assert_eq!(
        function_from_definition(
            "arr()",
            "CREATE FUNCTION arr() RETURNS integer[] LANGUAGE sql AS $$ SELECT 1 $$",
        )
        .return_contract,
        "integer[]"
    );
    assert_eq!(
        function_from_definition(
            "safe()",
            "CREATE FUNCTION safe() RETURNS int LANGUAGE sql PARALLEL SAFE AS $$ SELECT 1 $$",
        )
        .parallel,
        "safe"
    );
    assert_eq!(
        function_from_definition(
            "typed()",
            "CREATE FUNCTION typed() RETURNS \"TypeA\" LANGUAGE sql AS $$ SELECT 1 $$",
        )
        .return_contract,
        "\"TypeA\""
    );
    assert_eq!(
        function_from_definition(
            "barrier()",
            "CREATE FUNCTION barrier() RETURNS int LANGUAGE sql LEAKPROOF AS $$ SELECT 1 $$",
        )
        .leakproof,
        "yes"
    );
    assert_eq!(
        function_from_definition(
            "open()",
            "CREATE FUNCTION open() RETURNS int NOT LEAKPROOF LANGUAGE sql AS $$ SELECT 1 $$",
        )
        .leakproof,
        "no"
    );
    let coded = function_from_definition(
        "coded()",
        "CREATE FUNCTION coded() RETURNS text LANGUAGE sql AS E'\\x31\\101\\u0041\\U00000042\\q'",
    );
    assert_eq!(coded.body.as_deref(), Some("1AABq"));
    let atomic_if = function_from_definition(
        "t()",
        "CREATE FUNCTION t() RETURNS int LANGUAGE sql BEGIN ATOMIC CREATE TABLE IF NOT EXISTS widgets(id int); SELECT 1; END",
    );
    assert!(atomic_if.body.unwrap().contains("SELECT 1"));
    let if_exists = function_from_definition(
        "e()",
        "CREATE FUNCTION e() RETURNS int LANGUAGE sql BEGIN ATOMIC DROP TABLE IF EXISTS widgets; SELECT 2; END",
    );
    assert!(if_exists.body.unwrap().contains("SELECT 2"));
    let if_block = function_from_definition(
        "b()",
        "CREATE FUNCTION b() RETURNS int LANGUAGE sql BEGIN ATOMIC IF 1 SELECT 3; END END",
    );
    assert!(if_block.body.unwrap().contains("SELECT 3"));
    let path_security = function_from_definition(
        "s()",
        "CREATE FUNCTION s() RETURNS int LANGUAGE sql SET search_path TO security, definer AS $$ SELECT 1 $$",
    );
    assert_eq!(path_security.security, "invoker");
    let outs = function_from_definition(
        "o(int,text)",
        "CREATE FUNCTION o(OUT a int, OUT b text) RETURNS record LANGUAGE sql AS $$ SELECT 1, 2 $$",
    );
    assert!(
        outs.return_contract.contains("out a int"),
        "{}",
        outs.return_contract
    );
    assert!(outs.return_contract.contains("out b text"));
    assert_eq!(
        function_from_definition(
            "plain()",
            "CREATE FUNCTION plain() RETURNS int LANGUAGE sql AS $$ SELECT 1 $$",
        )
        .parallel,
        "unsafe"
    );
    assert_eq!(
        function_from_definition(
            "limited()",
            "CREATE FUNCTION limited() RETURNS int LANGUAGE sql PARALLEL RESTRICTED AS $$ SELECT 1 $$",
        )
        .parallel,
        "restricted"
    );
    assert_eq!(
        function_from_definition(
            "marked()",
            "CREATE FUNCTION marked() RETURNS int LANGUAGE sql PARALLEL SAFE PARALLEL UNSAFE AS $$ SELECT 1 $$",
        )
        .parallel,
        "unsafe"
    );
}

#[test]
fn parameter_names_are_not_modes_and_defensive_clauses_parse() {
    let named = function_from_definition(
        "fn_param(integer)",
        "CREATE FUNCTION fn_param(immutable int) RETURNS int LANGUAGE sql AS $$ SELECT 1 $$",
    );
    assert_eq!(named.volatility, "volatile");
    assert_eq!(
        function_from_definition(
            "fn_real()",
            "CREATE FUNCTION fn_real() RETURNS int IMMUTABLE LANGUAGE sql AS $$ SELECT 1 $$",
        )
        .volatility,
        "immutable"
    );
    assert_eq!(
        function_from_definition(
            "fn_nested(integer)",
            "CREATE FUNCTION fn_nested(a int DEFAULT (1)) RETURNS int IMMUTABLE LANGUAGE sql AS $$ SELECT 1 $$",
        )
        .volatility,
        "immutable"
    );
    assert_eq!(
        function_from_definition(
            "fn_comment(integer)",
            "CREATE FUNCTION fn_comment(a int /* (immutable) */) RETURNS int IMMUTABLE LANGUAGE sql AS $$ SELECT 1 $$",
        )
        .volatility,
        "immutable"
    );
    assert_eq!(
        function_from_definition(
            "fn_called()",
            "CREATE FUNCTION fn_called() RETURNS int STRICT CALLED ON NULL INPUT LANGUAGE sql AS $$ SELECT 1 $$",
        )
        .null_input,
        "called"
    );
    assert_eq!(
        function_from_definition(
            "fn_invoker()",
            "CREATE FUNCTION fn_invoker() RETURNS int SECURITY DEFINER SECURITY INVOKER LANGUAGE sql AS $$ SELECT 1 $$",
        )
        .security,
        "invoker"
    );
    assert_eq!(
        function_from_definition(
            "fn_nulls()",
            "CREATE FUNCTION fn_nulls() RETURNS NULL ON NULL INPUT LANGUAGE sql AS $$ SELECT 1 $$",
        )
        .null_input,
        "strict"
    );
    let outputs = function_from_definition(
        "fn_out(integer)",
        "CREATE FUNCTION fn_out(OUT a int) RETURNS LANGUAGE sql AS $$ SELECT 1 $$",
    );
    assert!(
        outputs.return_contract.contains("out a int"),
        "{}",
        outputs.return_contract
    );
    assert_eq!(
        function_from_definition(
            "fn_gap()",
            "CREATE FUNCTION fn_gap() RETURNS int LANGUAGE sql AS /* note */ $$ SELECT 3 $$",
        )
        .body
        .as_deref(),
        Some(" SELECT 3 ")
    );
    assert!(function_from_definition(
        "fn_atomic()",
        "CREATE FUNCTION fn_atomic() RETURNS int LANGUAGE sql BEGIN ATOMIC /* c */ SELECT 4; END",
    )
    .body
    .is_some());
    assert!(function_from_definition(
        "fn_open()",
        "CREATE FUNCTION fn_open() RETURNS int LANGUAGE sql BEGIN ATOMIC SELECT 5",
    )
    .body
    .is_none());
    assert_eq!(
        function_from_definition(
            "fn_lang()",
            "CREATE FUNCTION fn_lang() RETURNS int LANGUAGE 'SQL' AS $$ SELECT 6 $$",
        )
        .language
        .as_deref(),
        Some("sql")
    );
    assert_eq!(
        function_from_definition(
            "fn_doubled()",
            "CREATE FUNCTION fn_doubled(x text DEFAULT E'it''s') RETURNS int LANGUAGE sql AS $$ SELECT 1 $$",
        )
        .body
        .as_deref(),
        Some(" SELECT 1 ")
    );
    assert_eq!(
        function_from_definition(
            "fn_hex()",
            "CREATE FUNCTION fn_hex() RETURNS int LANGUAGE sql AS E'\\u12'",
        )
        .body
        .as_deref(),
        Some("u12")
    );
    assert_eq!(
        function_from_definition(
            "fn_octal()",
            "CREATE FUNCTION fn_octal() RETURNS int LANGUAGE sql AS E'\\18'",
        )
        .body
        .as_deref(),
        Some("\u{1}8")
    );
    assert!(function_from_definition(
        "fn_unclosed()",
        "CREATE FUNCTION fn_unclosed(x text DEFAULT E'unterminated",
    )
    .body
    .is_none());
    assert!(function_from_definition(
        "fn_plain()",
        "CREATE FUNCTION fn_plain() RETURNS int LANGUAGE sql AS 'unterminated",
    )
    .body
    .is_none());
    assert_eq!(
        function_from_definition(
            "fn_bare()",
            "CREATE FUNCTION fn_bare RETURNS int IMMUTABLE LANGUAGE sql AS $$ SELECT 1 $$",
        )
        .volatility,
        "immutable"
    );
    assert_eq!(super::super::function_body::skip_as_gap("abc", 10), 10);
    assert_eq!(
        super::super::function_body::language_name("  sql", 0).as_deref(),
        Some("sql")
    );
    assert_eq!(
        super::super::function_clauses::header_modes(
            "CREATE FUNCTION f() RETURNS int IMMUTABLE",
            Some((8, 2)),
        )
        .volatility,
        "immutable"
    );
    assert_eq!(
        super::super::function_clauses::header_modes("[]", None).volatility,
        "volatile"
    );
    assert_eq!(
        super::super::function_comment::skip_escape_string("noteE'abc'", 5),
        None
    );
    assert_eq!(
        super::super::function_comment::skip_escape_string("E'unterminated", 1),
        Some("E'unterminated".len())
    );
    assert_eq!(
        super::super::function_quote::opening_dollar("a$tag$", 1),
        None
    );
    assert_eq!(super::super::function_quote::opening_dollar("$1$", 0), None);
    assert_eq!(
        super::super::function_quote::quoted_sql_body("abc", 0),
        None
    );
    assert_eq!(
        super::super::function_outputs::after_parameter_list("CREATE FUNCTION f RETURNS int"),
        "CREATE FUNCTION f RETURNS int"
    );
    let alpha = function_from_definition(
        "fn_alpha()",
        "CREATE FUNCTION fn_alpha() RETURNS α LANGUAGE sql AS $$ SELECT 1 $$",
    );
    let beta = function_from_definition(
        "fn_beta()",
        "CREATE FUNCTION fn_beta() RETURNS β LANGUAGE sql AS $$ SELECT 1 $$",
    );
    assert!(
        alpha.return_contract.contains('α'),
        "{}",
        alpha.return_contract
    );
    assert!(
        beta.return_contract.contains('β'),
        "{}",
        beta.return_contract
    );
    assert_ne!(alpha.return_contract, beta.return_contract);
}
