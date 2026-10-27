use super::*;

fn settings(identifiers: bool) -> normalize::Settings {
    normalize::Settings {
        normalize_identifiers: identifiers,
        normalize_raise: true,
        keep_identifiers: Vec::new(),
    }
}

#[test]
fn search_path_stops_at_volatility_and_skips_noise() {
    let path = search_path::extract(
        "CREATE FUNCTION f() SET work_mem TO x SET search_path TO /* nest /* in */ */ tenant_a -- note\n, \"ten\"\"ant\", $1$ IMMUTABLE",
    );
    assert!(path.contains("tenant_a"), "{path}");
    assert!(path.contains("\"ten\"\"ant\""), "{path}");
    assert!(!path.contains("immutable"), "{path}");
    assert_eq!(
        search_path::extract("SET search_path TO stable"),
        "to stable"
    );
    assert_eq!(
        search_path::extract("SET search_path TO immutable"),
        "to immutable"
    );
    assert!(!search_path::extract("SET search_path TO tenant_a STABLE").contains("stable"));
    assert!(!search_path::extract("SET search_path TO tenant_a VOLATILE").contains("volatile"));
    let stopped = search_path::extract("SET search_path TO tenant_a; LANGUAGE sql");
    assert!(!stopped.contains("language"), "{stopped}");
    let commented = search_path::extract("SET -- note\n search_path TO public");
    assert!(commented.contains("public"), "{commented}");
    let unicode = search_path::extract("SET search_path TO café, public");
    assert!(unicode.contains("café"), "{unicode}");
    assert!(unicode.contains("public"), "{unicode}");
}

#[test]
fn qualified_raise_calls_stay_calls() {
    let tokens = normalize::normalized_tokens(
        "PERFORM public.raise(send_email())",
        &settings(true),
        Some("plpgsql"),
    )
    .unwrap();
    assert!(tokens.iter().any(|token| token == "raise"), "{tokens:?}");
    assert!(!tokens.iter().any(|token| token == "RAISE"), "{tokens:?}");
}

#[test]
fn normalization_covers_closed_calls_and_quoted_names() {
    let nested = normalize::normalized_tokens(
        "SELECT helper(g(1), (widget) := 2)",
        &settings(true),
        Some("sql"),
    )
    .unwrap();
    assert!(nested.iter().any(|token| token == "helper"));
    let quoted =
        normalize::normalized_tokens("SELECT \"widget\"", &settings(true), Some("sql")).unwrap();
    assert!(quoted.iter().any(|token| token.starts_with("ID")));
}

#[test]
fn outside_body_keeps_text_when_the_span_is_unusable() {
    assert_eq!(scan::outside_body("abcdef", None), "abcdef");
    assert_eq!(scan::outside_body("abcdef", Some((4, 1))), "abcdef");
}
