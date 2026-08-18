use super::extract::{extract_named_destinations, ExtractedDestinations};
use super::options::Options;
use super::routes::{
    build_route_set, destination_matches, is_page_file, matches_route_segments,
    route_from_page_relative, should_skip_destination, strip_query_and_hash,
};
use super::scan::{check_named_section, contains_word, word_line};
use super::*;
use crate::config::v2::{
    schema::{Project, RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/nextjs-redirect-destinations/fixture")
            .join(name),
    )
}

fn config(options_yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(options_yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn fixture_files(root: &Path) -> Vec<PathBuf> {
    crate::codebase::ts_source::walk_files(root, &[])
}

fn run(name: &str, options_yaml: &str) -> Vec<RuleFinding> {
    let root = fixture(name);
    check_with_files(&root, &config(options_yaml), &fixture_files(&root)).unwrap()
}

fn messages(findings: &[RuleFinding]) -> Vec<&str> {
    findings
        .iter()
        .map(|finding| finding.message.as_str())
        .collect()
}

#[test]
fn pass_fixture_accepts_existing_page_and_skips_external_destinations() {
    let findings = run("pass", "{}");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn pass_group_unwraps_groups_and_slots() {
    assert!(run("pass-group", "{}").is_empty());
}

#[test]
fn fail_missing_flags_unknown_destination() {
    let findings = run("fail-missing", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].rule, RULE_ID);
    assert_eq!(findings[0].file, "next.config.ts");
    assert!(findings[0].message.contains("redirect destination '/gone'"));
    assert!(findings[0].message.contains("app/**/page.tsx"));
}

#[test]
fn skip_private_flags_underscore_segment_pages() {
    let findings = run("skip-private", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0]
        .message
        .contains("redirect destination '/secret'"));
}

#[test]
fn dynamic_matches_slug_and_catch_all_routes() {
    assert!(run("dynamic", "{}").is_empty());
}

#[test]
fn rewrite_fail_flags_nested_rewrite_destinations() {
    let findings = run("rewrite-fail", "{}");
    let body = messages(&findings).join("\n");
    assert_eq!(findings.len(), 3, "{findings:?}");
    assert!(body.contains("rewrite destination '/missing-before'"));
    assert!(body.contains("rewrite destination '/missing-after'"));
    assert!(body.contains("rewrite destination '/missing-fallback'"));
}

#[test]
fn include_rewrites_false_skips_rewrite_destinations() {
    let findings = run("rewrite-fail", "{ includeRewrites: false }");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn fail_extractor_flags_unparsed_redirects_and_rewrites() {
    let findings = run("fail-extractor", "{}");
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings.iter().any(|finding| finding
        .message
        .contains("could not locate the redirects() body")));
    assert!(findings.iter().any(|finding| finding
        .message
        .contains("could not locate the rewrites() body")));
}

#[test]
fn dest_non_string_flags_extractor_destination_drift() {
    let findings = run("dest-non-string", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0]
        .message
        .contains("extracted no string destinations"));
}

#[test]
fn custom_config_path_and_app_root() {
    let findings = run(
        "custom-app-root",
        "{ configPath: configs/next.config.ts, appRoot: src/app }",
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn next_config_mjs_and_js_are_discovered() {
    assert!(run("config-mjs", "{}").is_empty());
    assert!(run("config-js", "{}").is_empty());
}

#[test]
fn disable_file_comment_skips_the_config() {
    assert!(run("disable-file", "{}").is_empty());
}

#[test]
fn missing_next_config_is_silent() {
    assert!(run("no-config", "{}").is_empty());
}

#[test]
fn empty_redirects_body_is_silent() {
    assert!(run("empty-redirects", "{}").is_empty());
}

#[test]
fn coverage_shapes_accepts_class_and_arrow_configs() {
    assert!(run("coverage-shapes", "{}").is_empty());
}

#[test]
fn check_and_public_dispatch_use_the_same_fixtures() {
    let root = fixture("fail-missing");
    let findings = check(&root, &config("{}")).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");

    let dispatched =
        crate::codebase::rules::run_filesystem_rules(&root, Some(&root.join(".no-mistakes.yml")))
            .unwrap();
    assert_eq!(dispatched.len(), 1, "{dispatched:?}");
}

#[test]
fn options_default_enables_rewrites() {
    assert!(Options::default().include_rewrites);
}

#[test]
fn blank_config_path_falls_back_to_default_names() {
    assert!(run("pass", "{ configPath: '   ' }").is_empty());
}

#[test]
fn absolute_config_path_is_resolved() {
    let root = fixture("pass");
    let abs = root.join("next.config.ts");
    let options = format!("{{ configPath: \"{}\" }}", abs.display());
    let findings = check_with_files(&root, &config(&options), &fixture_files(&root)).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn unknown_config_path_is_silent() {
    assert!(run("pass", "{ configPath: missing.config.ts }").is_empty());
}

#[test]
fn unreadable_config_path_is_silent() {
    let root = fixture("pass");
    let missing = root.join("nope.config.ts");
    let findings =
        check_with_files(&root, &config("{ configPath: nope.config.ts }"), &[missing]).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn blank_app_root_uses_default_app_directory() {
    let findings = run("fail-missing", "{ appRoot: ' ./ ' }");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("expected app/**/page.tsx"));
}

#[test]
fn app_root_label_normalizes_slashes() {
    let findings = run("fail-missing", r#"{ appRoot: "src\\app" }"#);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("expected src/app/**/page.tsx"));
}

#[test]
fn invalid_include_glob_is_an_error() {
    let root = fixture("pass");
    let mut config = config("{}");
    config.rules[0].include = vec!["[".to_string()];
    let error = check_with_files(&root, &config, &fixture_files(&root)).unwrap_err();
    assert!(
        error.to_string().contains("include contains invalid glob"),
        "{error:#}"
    );
}

#[test]
fn project_scope_uses_configured_root_not_web() {
    let root = fixture("custom-app-root");
    let mut config = NoMistakesConfig::default();
    config.projects.insert(
        "frontend".to_string(),
        Project {
            root: Some(".".to_string()),
            ..Default::default()
        },
    );
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        projects: vec!["frontend".to_string()],
        options: serde_yaml::from_str("{ configPath: configs/next.config.ts, appRoot: src/app }")
            .unwrap(),
        ..Default::default()
    });
    let findings = check_with_files(&root, &config, &fixture_files(&root)).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn substring_redirects_text_is_not_extractor_drift() {
    let extracted = extract_named_destinations(
        Path::new("next.config.ts"),
        "export default { preredirects() { return []; } };\n",
        "redirects",
    );
    assert!(!extracted.body_found);
    let findings = check_named_section(
        "next.config.ts",
        "export default { preredirects() { return []; } };\n",
        Path::new("next.config.ts"),
        "redirects",
        "redirect",
        &BTreeSet::new(),
        "app",
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn extract_reads_object_methods_arrows_and_class_members() {
    let object = extract_named_destinations(
        Path::new("next.config.ts"),
        r#"
const nextConfig = {
  other: { keep: true },
  async redirects() {
    return [{ source: "/old", destination: "/about", permanent: true }];
  },
};
export default nextConfig;
"#,
        "redirects",
    );
    assert!(object.body_found);
    assert_eq!(
        object
            .destinations
            .iter()
            .map(|destination| destination.value.as_str())
            .collect::<Vec<_>>(),
        ["/about"]
    );

    let arrow = extract_named_destinations(
        Path::new("next.config.ts"),
        r#"
export default {
  redirects: () => [{ destination: "/login" }],
};
"#,
        "redirects",
    );
    assert!(arrow.body_found);
    assert_eq!(arrow.destinations[0].value, "/login");

    let function_expr = extract_named_destinations(
        Path::new("next.config.ts"),
        r#"
export default {
  redirects: function () {
    return [{ destination: "/docs" }];
  },
};
"#,
        "redirects",
    );
    assert!(function_expr.body_found);
    assert_eq!(function_expr.destinations[0].value, "/docs");

    let class_source = r#"
class Config {
  other() {}
  extra = 1;
  async redirects() {
    return [{ destination: "/about" }];
  }
  rewrites = async () => [{ destination: "/login" }];
}
"#;
    let class_redirects =
        extract_named_destinations(Path::new("next.config.ts"), class_source, "redirects");
    let class_rewrites =
        extract_named_destinations(Path::new("next.config.ts"), class_source, "rewrites");
    assert!(class_redirects.body_found);
    assert!(class_rewrites.body_found);
    assert_eq!(class_redirects.destinations[0].value, "/about");
    assert_eq!(class_rewrites.destinations[0].value, "/login");
}

#[test]
fn extract_walks_nested_objects_and_ignores_spreads() {
    let nested = extract_named_destinations(
        Path::new("next.config.ts"),
        r#"
export default {
  nested: {
    async redirects() {
      return [
        { ...extra },
        { destination: `/dynamic` },
        { destination: "/about" as const },
      ];
    },
  },
};
"#,
        "redirects",
    );
    assert!(nested.body_found);
    assert!(nested.saw_destination_property);
    assert_eq!(
        nested
            .destinations
            .iter()
            .map(|destination| destination.value.as_str())
            .collect::<Vec<_>>(),
        ["/about"]
    );
}

#[test]
fn extract_keeps_the_first_matching_body() {
    let extracted = extract_named_destinations(
        Path::new("next.config.ts"),
        r#"
export default {
  async redirects() {
    return [{ destination: "/first" }];
  },
  extra: {
    async redirects() {
      return [{ destination: "/second" }];
    },
  },
};
"#,
        "redirects",
    );
    assert_eq!(
        extracted
            .destinations
            .iter()
            .map(|destination| destination.value.as_str())
            .collect::<Vec<_>>(),
        ["/first"]
    );
}

#[test]
fn extract_returns_default_on_parse_failure() {
    let extracted =
        extract_named_destinations(Path::new("next.config.ts"), "not { valid", "redirects");
    assert_eq!(extracted, ExtractedDestinations::default());
}

#[test]
fn route_helpers_build_and_match_app_router_paths() {
    assert!(is_page_file(Path::new("app/about/page.tsx")));
    assert!(is_page_file(Path::new("app/about/page.ts")));
    assert!(is_page_file(Path::new("app/about/page.jsx")));
    assert!(is_page_file(Path::new("app/about/page.js")));
    assert!(!is_page_file(Path::new("app/about/layout.tsx")));
    assert!(!is_page_file(Path::new("app/about/page.mdx")));

    assert_eq!(
        route_from_page_relative(Path::new("page.tsx")).as_deref(),
        Some("/")
    );
    assert_eq!(
        route_from_page_relative(Path::new("(auth)/login/page.tsx")).as_deref(),
        Some("/login")
    );
    assert_eq!(
        route_from_page_relative(Path::new("@modal/settings/page.tsx")).as_deref(),
        Some("/settings")
    );
    assert_eq!(
        route_from_page_relative(Path::new("_secret/page.tsx")),
        None
    );
    assert_eq!(
        route_from_page_relative(Path::new("blog/_draft/page.tsx")),
        None
    );
    assert_eq!(
        route_from_page_relative(Path::new("about/layout.tsx")),
        None
    );
    assert_eq!(
        route_from_page_relative(Path::new("a/../b/page.tsx")).as_deref(),
        Some("/a/b")
    );

    let routes = build_route_set(
        &[
            PathBuf::from("/repo/app/page.tsx"),
            PathBuf::from("/repo/app/about/page.tsx"),
            PathBuf::from("/repo/app/_secret/page.tsx"),
            PathBuf::from("/repo/app/blog/[slug]/page.tsx"),
            PathBuf::from("/repo/src/app/ignored/page.tsx"),
            PathBuf::from("/repo/app/about/layout.tsx"),
        ],
        Path::new("/repo/app"),
    );
    assert_eq!(
        routes,
        BTreeSet::from([
            "/".to_string(),
            "/about".to_string(),
            "/blog/[slug]".to_string()
        ])
    );

    assert_eq!(strip_query_and_hash("/about?x=1#top"), "/about");
    assert_eq!(strip_query_and_hash("/about#top"), "/about");
    assert!(should_skip_destination("https://example.com"));
    assert!(should_skip_destination("//cdn.example.com"));
    assert!(should_skip_destination("/:slug"));
    assert!(!should_skip_destination("/about"));

    assert!(destination_matches(&routes, "/about"));
    assert!(destination_matches(&routes, "/blog/hello"));
    assert!(!destination_matches(&routes, "/gone"));
    assert!(destination_matches(&routes, "/"));
    assert!(!destination_matches(&routes, "/missing"));

    assert!(matches_route_segments(
        &["blog", "[slug]"],
        &["blog", "hello"]
    ));
    assert!(!matches_route_segments(
        &["blog", "[slug]"],
        &["blog", "hello", "extra"]
    ));
    assert!(matches_route_segments(
        &["docs", "[...slug]"],
        &["docs", "a", "b"]
    ));
    assert!(!matches_route_segments(&["docs", "[...slug]"], &["docs"]));
    assert!(matches_route_segments(
        &["optional", "[[...slug]]"],
        &["optional"]
    ));
    assert!(matches_route_segments(&["[...all]"], &["a"]));
}

#[test]
fn word_helpers_require_standalone_tokens() {
    assert!(contains_word("async redirects() {}", "redirects"));
    assert!(!contains_word("preredirects() {}", "redirects"));
    assert!(!contains_word("redirectsExtra() {}", "redirects"));
    assert_eq!(
        word_line("const x = 1;\nasync redirects() {}", "redirects"),
        2
    );
    assert_eq!(word_line("nope", "redirects"), 1);
}
