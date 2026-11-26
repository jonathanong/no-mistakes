use super::{check_with_files, diagnostics_with_files, CspDirective, CspPolicySet, ParsedOrigin};
use std::path::PathBuf;

#[test]
fn parses_and_matches_exact_wildcard_scheme_and_fallback_sources() {
    let policy = CspPolicySet::parse_from_strings(&[[
        "default-src 'self' https://fallback.example.com",
        "img-src 'self' https://cdn.example.com:443 https://*.assets.example.org https://[2001:db8::1]",
        "connect-src 'self' wss://rt.example.com:8443 ws://ws.example.com:* http://[::1]:3000 https: 'nonce-abc'",
        "frame-src 'none'",
        "media-src *",
        "font-src fonts.example.com localhost:3000",
    ]
    .join("; ")]);
    let self_origins = vec![ParsedOrigin::parse("https://app.example.com").unwrap()];

    assert!(policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://cdn.example.com/logo.png").unwrap(),
        &self_origins,
    ));
    assert!(policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("//cdn.example.com/proto-rel.png").unwrap(),
        &self_origins,
    ));
    assert!(policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://[2001:0db8::1]/v6.png").unwrap(),
        &self_origins,
    ));
    assert!(policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("//[2001:db8::1]:443/v6.png").unwrap(),
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
        &ParsedOrigin::parse("http://[::1]:3000/api").unwrap(),
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
fn models_duplicate_directives_and_policy_intersection() {
    let dup = CspPolicySet::parse_from_strings(&[
        "default-src 'self'; script-src 'self'; script-src https://second.example.com".into(),
    ]);
    assert!(!dup.allows(
        CspDirective::Script,
        &ParsedOrigin::parse("https://second.example.com/a.js").unwrap(),
        &[],
    ));

    let separate = CspPolicySet::parse_from_strings(&[
        "default-src 'self'; script-src 'self' https://shared.example.com https://dev.example.com"
            .into(),
        "default-src 'self'; script-src 'self' https://shared.example.com".into(),
    ]);
    assert!(separate.allows(
        CspDirective::Script,
        &ParsedOrigin::parse("https://shared.example.com/a.js").unwrap(),
        &[],
    ));
    assert!(!separate.allows(
        CspDirective::Script,
        &ParsedOrigin::parse("https://dev.example.com/a.js").unwrap(),
        &[],
    ));

    let comma_joined = CspPolicySet::parse_from_text(
        "Content-Security-Policy: script-src 'self' https://dev.example.com, script-src 'self'",
    );
    assert!(!comma_joined.allows(
        CspDirective::Script,
        &ParsedOrigin::parse("https://dev.example.com/a.js").unwrap(),
        &[],
    ));
    assert!(comma_joined.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://unrestricted.example.com/a.png").unwrap(),
        &[],
    ));
    let invalid_sources =
        CspPolicySet::parse_from_text("Content-Security-Policy: img-src ftp: https://[::1]junk");
    assert!(!invalid_sources.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://cdn.example.com/a.png").unwrap(),
        &[],
    ));
}

#[test]
fn parses_unquoted_yaml_block_scalars_and_single_quoted_meta_without_scheme_split() {
    let yaml = "content_security_policy: |\n  default-src 'self';\n  img-src 'self' https://cdn.example.com;\n  connect-src http://[::1]:* wss://rt.example.com;\n";
    let policy = CspPolicySet::parse_from_text(yaml);
    assert!(policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://cdn.example.com/a.png").unwrap(),
        &[],
    ));
    assert!(!policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://other.example.net/a.png").unwrap(),
        &[],
    ));
    assert!(policy.allows(
        CspDirective::Connect,
        &ParsedOrigin::parse("http://[::1]:4000/api").unwrap(),
        &[],
    ));

    let html = "<meta http-equiv='Content-Security-Policy' content='default-src https://meta.example.com; img-src https://img.example.com'>";
    let html_policy = CspPolicySet::parse_from_text(html);
    assert!(html_policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://img.example.com/a.png").unwrap(),
        &[],
    ));
    assert!(!html_policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://unlisted.example.net/a.png").unwrap(),
        &[],
    ));

    let mixed_double_then_single = "<meta http-equiv=\"Content-Security-Policy\" content='default-src https://meta.example.com; img-src https://img.example.com'>";
    let mixed_policy = CspPolicySet::parse_from_text(mixed_double_then_single);
    assert!(mixed_policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://img.example.com/a.png").unwrap(),
        &[],
    ));
    assert!(!mixed_policy.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://unlisted.example.net/a.png").unwrap(),
        &[],
    ));

    let mixed_single_then_double = "<meta http-equiv='Content-Security-Policy' content=\"default-src 'self'; img-src https://img.example.com\">";
    let mixed_reverse = CspPolicySet::parse_from_text(mixed_single_then_double);
    assert!(mixed_reverse.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://img.example.com/a.png").unwrap(),
        &[],
    ));
    assert!(!mixed_reverse.allows(
        CspDirective::Img,
        &ParsedOrigin::parse("https://unlisted.example.net/a.png").unwrap(),
        &[],
    ));
}

#[test]
fn handles_child_src_fallback_text_comments_and_missing_directives() {
    let headers = "# comment\n// js comment\n\"default-src 'self'; child-src https://frames.example.com; ; report-uri /r; script-src-elem http: data: ftp://bad.example.com bareword https://js.example.com; style-src-elem https://css.example.com\"\n\"unclosed\n";
    let policy = CspPolicySet::parse_from_text(headers);
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
    assert_eq!(CspDirective::ScriptElem.as_str(), "script-src-elem");
    assert_eq!(CspDirective::StyleElem.as_str(), "style-src-elem");
    assert_eq!(
        ParsedOrigin::parse("http://example.com:80/x")
            .unwrap()
            .display(),
        "http://example.com"
    );
    assert_eq!(
        ParsedOrigin::parse("http://example.com:8080/x")
            .unwrap()
            .display(),
        "http://example.com:8080"
    );
    assert_eq!(
        ParsedOrigin::parse("//cdn.example.com/a.js")
            .unwrap()
            .display(),
        "https://cdn.example.com"
    );
    assert_eq!(
        ParsedOrigin::parse("http://[::1]:3000/a")
            .unwrap()
            .display(),
        "http://[::1]:3000"
    );
}

#[test]
fn ignores_relative_and_non_network_urls_and_flags_malformed_network_urls() {
    assert!(ParsedOrigin::parse("/api/v1/items").is_none());
    assert!(!ParsedOrigin::is_network_like_url("/api/v1/items"));
    assert!(ParsedOrigin::parse("./local.js").is_none());
    assert!(ParsedOrigin::parse("data:image/png;base64,AAA").is_none());
    assert!(ParsedOrigin::parse("blob:https://app.example.com/123").is_none());
    assert!(ParsedOrigin::parse("mailto:user@example.com").is_none());
    assert!(ParsedOrigin::parse("https://user@example.com/x").is_none());
    assert!(ParsedOrigin::is_network_like_url(
        "https://user@example.com/x"
    ));
    assert!(ParsedOrigin::parse("https://example.com:notaport/x").is_none());
    assert!(ParsedOrigin::parse("https://bad host/x").is_none());
    assert!(ParsedOrigin::parse("https://[not-ipv6]/x").is_none());
    assert!(ParsedOrigin::parse("https://[::1/x").is_none());
    assert!(ParsedOrigin::parse("https://[::1]junk/x").is_none());
    assert!(ParsedOrigin::parse("https://bad]host/x").is_none());
    assert!(ParsedOrigin::parse("https://a:b:80/x").is_none());
    assert!(ParsedOrigin::parse("///empty-host").is_none());
    assert!(ParsedOrigin::is_network_like_url("///empty-host"));
    assert!(
        CspPolicySet::parse_from_text(":empty-prefix\nhttps://just-url\ncontent='unclosed")
            .is_empty()
    );
}

#[test]
fn reports_invalid_options_unreadable_csp_files_and_unanalyzable_modes() {
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

    let unanalyzable_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/csp-allowed-origins/fixture/unanalyzable-and-urls");
    let u_files = vec![
        unanalyzable_root.join("src/csp.ts"),
        unanalyzable_root.join("src/page.tsx"),
    ];
    let advise_cfg = crate::config::v2::load_v2_config_with_path(
        &unanalyzable_root,
        Some(&unanalyzable_root.join("advise.yml")),
    )
    .unwrap()
    .0;
    let diag = diagnostics_with_files(&unanalyzable_root, &advise_cfg, &u_files).unwrap();
    assert_eq!(diag.findings.len(), 2);
    assert_eq!(diag.advisories.len(), 9);
    assert!(diag
        .advisories
        .iter()
        .all(|item| item.target.as_deref() == Some("unanalyzable")));

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

#[test]
fn covers_bare_arrays_binary_concat_and_expression_edge_cases() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/csp-allowed-origins/fixture/js-policy-forms");
    for (cfg_name, csp_file) in [
        ("disjoint.yml", "src/bare-disjoint.ts"),
        ("overlap.yml", "src/bare-overlap.ts"),
        ("concat.yml", "src/concat.ts"),
    ] {
        let cfg = crate::config::v2::load_v2_config_with_path(&root, Some(&root.join(cfg_name)))
            .unwrap()
            .0;
        let files = vec![root.join(csp_file), root.join("src/page.tsx")];
        let diag = diagnostics_with_files(&root, &cfg, &files).unwrap();
        assert!(
            diag.findings
                .iter()
                .any(|f| f.message.contains("https://dev.example.com")),
            "{cfg_name}: expected dev.example.com violation, got {:?}",
            diag.findings
        );
        assert!(
            !diag
                .findings
                .iter()
                .any(|f| f.message.contains("script-src")
                    && f.message.contains("https://shared.example.com")),
            "{cfg_name}: shared.example.com should be allowed for script-src, got {:?}",
            diag.findings
        );
        assert_eq!(
            diag.advisories.len(),
            4,
            "{cfg_name}: expected 4 advisories, got {:?}",
            diag.advisories
        );
    }

    let errors_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/csp-allowed-origins/fixture/errors");
    let broken_target_cfg = crate::config::v2::load_v2_config_with_path(
        &errors_root,
        Some(&errors_root.join("broken-target.yml")),
    )
    .unwrap()
    .0;
    let broken_findings = check_with_files(
        &errors_root,
        &broken_target_cfg,
        &[errors_root.join("broken.ts")],
    )
    .unwrap();
    assert_eq!(broken_findings.len(), 1);
    assert_eq!(broken_findings[0].file, "broken.ts");
    assert_eq!(broken_findings[0].line, 1);
    assert_eq!(broken_findings[0].target.as_deref(), Some("unanalyzable"));

    let parse_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/csp-allowed-origins/fixture/parse-error-target");
    let parse_files = vec![
        parse_root.join("src/csp.ts"),
        parse_root.join("src/broken.ts"),
    ];
    let advise_parse_cfg = crate::config::v2::load_v2_config_with_path(
        &parse_root,
        Some(&parse_root.join("advise.yml")),
    )
    .unwrap()
    .0;
    let advise_parse =
        diagnostics_with_files(&parse_root, &advise_parse_cfg, &parse_files).unwrap();
    assert!(advise_parse.findings.is_empty());
    assert_eq!(advise_parse.advisories.len(), 1);
    assert_eq!(
        advise_parse.advisories[0].target.as_deref(),
        Some("unanalyzable")
    );

    let ignore_parse_cfg = crate::config::v2::load_v2_config_with_path(
        &parse_root,
        Some(&parse_root.join("ignore.yml")),
    )
    .unwrap()
    .0;
    let ignore_parse =
        diagnostics_with_files(&parse_root, &ignore_parse_cfg, &parse_files).unwrap();
    assert!(ignore_parse.findings.is_empty());
    assert!(ignore_parse.advisories.is_empty());

    let headers_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/csp-allowed-origins/fixture/headers-text");
    let headers_cfg = crate::config::v2::load_v2_config_with_path(
        &headers_root,
        Some(&headers_root.join(".no-mistakes.yml")),
    )
    .unwrap()
    .0;
    let header_files = vec![headers_root.join("src/page.tsx")];
    let header_sources = crate::codebase::rules::source_store_for_files(&header_files);
    assert!(super::check_prepared(
        &headers_root,
        &headers_cfg,
        &header_files,
        None,
        &header_sources,
        None
    )
    .is_err());

    let mut custom_cfg =
        crate::config::v2::load_v2_config_with_path(&root, Some(&root.join("concat.yml")))
            .unwrap()
            .0;
    custom_cfg.rules[0].message = Some("Custom CSP hint".into());
    let custom_files = vec![root.join("src/concat.ts"), root.join("src/page.tsx")];
    let custom_diag = diagnostics_with_files(&root, &custom_cfg, &custom_files).unwrap();
    assert!(custom_diag
        .findings
        .iter()
        .chain(&custom_diag.advisories)
        .all(|item| item.message == "Custom CSP hint"));
}

mod dynamic_policies;
