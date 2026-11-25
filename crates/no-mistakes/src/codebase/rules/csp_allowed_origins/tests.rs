use super::{check_with_files, CspDirective, CspPolicy, ParsedOrigin};
use std::path::PathBuf;

#[test]
fn parses_and_matches_exact_wildcard_scheme_and_fallback_sources() {
    let policy = CspPolicy::parse_from_strings(&[
        "default-src 'self' https://fallback.example.com".into(),
        "img-src 'self' https://cdn.example.com:443 https://*.assets.example.org".into(),
        "connect-src 'self' wss://rt.example.com:8443 ws://ws.example.com:* https: 'nonce-abc'".into(),
        "frame-src 'none'".into(),
        "media-src *".into(),
        "font-src fonts.example.com localhost:3000".into(),
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
    assert!(policy.allows(
        CspDirective::Connect,
        &ParsedOrigin::parse("wss://ws.example.com:9000/stream").unwrap(),
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
    assert!(policy.allows(
        CspDirective::Media,
        &ParsedOrigin::parse("https://any.example.net/video.mp4").unwrap(),
        &self_origins,
    ));
    assert!(policy.allows(
        CspDirective::Font,
        &ParsedOrigin::parse("http://localhost:3000/f.woff2").unwrap(),
        &self_origins,
    ));
}

#[test]
fn handles_child_src_fallback_text_comments_and_missing_directives() {
    let headers = "# comment\n// js comment\n\"child-src https://frames.example.com; ; report-uri /r; script-src-elem http: data: ftp://bad.example.com bareword , https://js.example.com; style-src-elem https://css.example.com\"\n\"unclosed\ndefault-src 'self'\n";
    let policy = CspPolicy::parse_from_text(headers);
    assert!(policy.allows(
        CspDirective::Frame,
        &ParsedOrigin::parse("https://frames.example.com/embed").unwrap(),
        &[],
    ));
    assert!(policy.allows(
        CspDirective::Script,
        &ParsedOrigin::parse("https://any-https.example.com/app.js").unwrap(),
        &[],
    ));
    assert!(policy.allows(
        CspDirective::Style,
        &ParsedOrigin::parse("https://css.example.com/app.css").unwrap(),
        &[],
    ));
    assert!(!policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://missing-default.example.com/a.png").unwrap(),
        &[],
    ));
    assert_eq!(CspDirective::Child.as_str(), "child-src");
    assert_eq!(CspDirective::Font.as_str(), "font-src");
    assert_eq!(CspDirective::Default.as_str(), "default-src");
    assert_eq!(
        ParsedOrigin::parse("http://example.com:80/x").unwrap().display(),
        "http://example.com"
    );
    assert_eq!(
        ParsedOrigin::parse("http://example.com:8080/x").unwrap().display(),
        "http://example.com:8080"
    );
}

#[test]
fn ignores_relative_and_non_network_urls() {
    assert!(ParsedOrigin::parse("/api/v1/items").is_none());
    assert!(ParsedOrigin::parse("./local.js").is_none());
    assert!(ParsedOrigin::parse("data:image/png;base64,AAA").is_none());
    assert!(ParsedOrigin::parse("blob:https://app.example.com/123").is_none());
    assert!(ParsedOrigin::parse("mailto:user@example.com").is_none());
    assert!(ParsedOrigin::parse("https://user@example.com/x").is_none());
    assert!(ParsedOrigin::parse("https://example.com:notaport/x").is_none());
    assert!(ParsedOrigin::parse("https://bad host/x").is_none());
}

#[test]
fn reports_invalid_options_and_unreadable_csp_files() {
    let errors_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/csp-allowed-origins/fixture/errors");
    for (cfg_name, expected) in [
        ("empty-csp-files.yml", "requires non-empty options.cspFiles"),
        ("blank-csp-file.yml", "must be nonempty paths"),
        ("invalid-self-origin.yml", "must be a valid origin"),
        ("missing-csp-text.yml", "failed to read CSP file"),
        ("empty-csp-directives.yml", "no static CSP directives found"),
        ("broken-ts-csp.yml", "Repair the CSP file"),
    ] {
        let cfg = crate::config::v2::load_v2_config_with_path(
            &errors_root,
            Some(&errors_root.join(cfg_name)),
        )
        .unwrap()
        .0;
        let err = check_with_files(&errors_root, &cfg, &[]).unwrap_err();
        assert!(
            err.to_string().contains(expected),
            "{cfg_name}: unexpected error: {err}"
        );
    }

    let report_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/csp-allowed-origins/fixture/report");
    let default_off = crate::config::v2::load_v2_config_with_path(
        &report_root,
        Some(&report_root.join("default-off.yml")),
    )
    .unwrap()
    .0;
    assert!(check_with_files(&report_root, &default_off, &[])
        .unwrap()
        .is_empty());

    let enabled = crate::config::v2::load_v2_config_with_path(
        &report_root,
        Some(&report_root.join(".no-mistakes.yml")),
    )
    .unwrap()
    .0;
    let files = vec![
        report_root.join("src/csp.ts"),
        report_root.join("src/violations.tsx"),
        report_root.join("src/valid.tsx"),
    ];
    let findings = check_with_files(&report_root, &enabled, &files).unwrap();
    assert_eq!(findings.len(), 7);

    let sources = crate::codebase::rules::source_store_for_files(&files);
    assert!(super::check_prepared(&report_root, &enabled, &files, None, &sources, None).is_err());

    let mut plan_a = crate::codebase::check_facts::CheckFactPlan::default();
    let plan_b = crate::codebase::check_facts::CheckFactPlan {
        csp_allowed_origins_paths: std::sync::Arc::new(
            [report_root.join("src/valid.tsx")].into_iter().collect(),
        ),
        ..Default::default()
    };
    plan_a.include(plan_b);
    assert!(plan_a
        .csp_allowed_origins_paths
        .contains(&report_root.join("src/valid.tsx")));
}
