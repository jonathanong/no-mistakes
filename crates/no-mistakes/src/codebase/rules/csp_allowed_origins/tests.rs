use super::{CspDirective, CspPolicy, ParsedOrigin};

#[test]
fn parses_and_matches_exact_wildcard_scheme_and_fallback_sources() {
    let policy = CspPolicy::parse_from_strings(&[
        "default-src 'self' https://fallback.example.com".into(),
        "img-src 'self' https://cdn.example.com https://*.assets.example.org".into(),
        "connect-src 'self' wss://rt.example.com:8443 https:".into(),
        "frame-src 'none'".into(),
    ]);
    let self_origins = vec![ParsedOrigin::parse("https://app.example.com").unwrap()];

    assert!(policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://cdn.example.com/logo.png").unwrap(),
        &self_origins,
    ));
    assert!(policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://eu.assets.example.org/hero.webp").unwrap(),
        &self_origins,
    ));
    assert!(!policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://assets.example.org/bare-root.webp").unwrap(),
        &self_origins,
    ));
    assert!(policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://app.example.com/icon.svg").unwrap(),
        &self_origins,
    ));
    assert!(!policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://unlisted.example.net/pixel.gif").unwrap(),
        &self_origins,
    ));
    assert!(policy.allows(
        CspDirective::Script,
        &ParsedOrigin::parse("https://fallback.example.com/sdk.js").unwrap(),
        &self_origins,
    ));
    assert!(policy.allows(
        CspDirective::Connect,
        &ParsedOrigin::parse("wss://rt.example.com:8443/stream").unwrap(),
        &self_origins,
    ));
    assert!(!policy.allows(
        CspDirective::Connect,
        &ParsedOrigin::parse("wss://rt.example.com:9443/stream").unwrap(),
        &self_origins,
    ));
    assert!(!policy.allows(
        CspDirective::Frame,
        &ParsedOrigin::parse("https://fallback.example.com/embed").unwrap(),
        &self_origins,
    ));
}

#[test]
fn parses_non_js_text_headers_and_json_values() {
    let headers = "Content-Security-Policy: default-src 'self'; script-src https://js.example.com\n";
    let policy = CspPolicy::parse_from_text(headers);
    assert!(policy.allows(
        CspDirective::Script,
        &ParsedOrigin::parse("https://js.example.com/app.js").unwrap(),
        &[],
    ));
    assert!(!policy.allows(
        CspDirective::Script,
        &ParsedOrigin::parse("https://evil.example.net/app.js").unwrap(),
        &[],
    ));
}

#[test]
fn ignores_relative_and_non_network_urls() {
    assert!(ParsedOrigin::parse("/api/v1/items").is_none());
    assert!(ParsedOrigin::parse("./local.js").is_none());
    assert!(ParsedOrigin::parse("data:image/png;base64,AAA").is_none());
    assert!(ParsedOrigin::parse("blob:https://app.example.com/123").is_none());
    assert!(ParsedOrigin::parse("mailto:user@example.com").is_none());
}
