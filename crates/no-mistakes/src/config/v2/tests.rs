use std::path::Path;

use super::discover::{find_config_root, load_v2_config};
use super::schema::{
    NoMistakesConfig, Project, ProjectType, RewriteRule, RuleDef, StringOrList, TestPlanPercent,
};
use super::view::ConfigView;

mod config_view;
mod test_plan_parse;

fn fixture(sub: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("fixtures/config-v2")
        .join(sub)
}

// ── discovery ─────────────────────────────────────────────────────────────────

#[test]
fn empty_config_returns_default() {
    let cfg = load_v2_config(&fixture("empty"), None).unwrap();
    assert_eq!(cfg, NoMistakesConfig::default());
}

#[test]
fn missing_dir_returns_default() {
    let cfg = load_v2_config(Path::new("/tmp/no-mistakes-nonexistent-xyz"), None).unwrap();
    assert_eq!(cfg, NoMistakesConfig::default());
}

#[test]
fn explicit_config_path_overrides_discovery() {
    let dir = fixture("multi-project");
    let explicit = dir.join(".no-mistakes.yml");
    let cfg = load_v2_config(&dir, Some(&explicit)).unwrap();
    assert!(cfg.projects.contains_key("web"));
}

#[test]
fn explicit_legacy_guardrails_path_parsed() {
    let dir = fixture("legacy-guardrails");
    let explicit = dir.join(".guardrailsrc.yml");
    let cfg = load_v2_config(&dir, Some(&explicit)).unwrap();
    assert!(cfg.projects.contains_key("backend"));
}

#[test]
fn explicit_nonexistent_config_errors() {
    let dir = fixture("basic");
    let err = load_v2_config(&dir, Some(Path::new("nonexistent.yml")))
        .err()
        .unwrap();
    assert!(err.to_string().contains("does not exist"));
}

// ── v2 format ─────────────────────────────────────────────────────────────────

#[test]
fn basic_v2_config_parsed() {
    let cfg = load_v2_config(&fixture("basic"), None).unwrap();
    let backend = &cfg.projects["backend"];
    assert_eq!(backend.type_, Some(ProjectType::Server));
    assert_eq!(backend.root.as_deref(), Some("backend"));
    assert!(cfg.rule_configured("http-route-static-paths"));
}

#[test]
fn multi_project_config_parsed() {
    let cfg = load_v2_config(&fixture("multi-project"), None).unwrap();
    assert_eq!(cfg.projects["web"].type_, Some(ProjectType::Nextjs));
    let queues = &cfg.projects["backend"].queues;
    assert_eq!(queues.enqueues, vec!["backend/queues/**"]);
    assert_eq!(queues.workers, vec!["backend/workers/**"]);
    assert_eq!(
        cfg.filesystem.skip_directories,
        vec![".next", "node_modules"]
    );
    let pw = &cfg.tests.playwright;
    assert!(matches!(&pw.configs, Some(StringOrList::One(s)) if s == "playwright.config.ts"));
    assert!(!pw.selectors.html_ids);
    assert_eq!(pw.selectors.test_ids, vec!["data-testid", "data-pw"]);
    assert_eq!(pw.selectors.component_test_ids["dataPw"], "data-pw");
    assert_eq!(pw.selector_roots, vec!["web/app", "web/components"]);
}

#[test]
fn nextjs_rewrites_parsed() {
    let cfg = load_v2_config(&fixture("nextjs-rewrites"), None).unwrap();
    let web = &cfg.projects["web"];
    assert_eq!(web.type_, Some(ProjectType::Nextjs));
    assert_eq!(web.rewrites.len(), 2);
    assert_eq!(
        web.rewrites[0],
        RewriteRule {
            source: "/posts/:slug*".to_string(),
            destination: "/content/posts/:slug*".to_string(),
        }
    );
    assert_eq!(
        web.rewrites[1],
        RewriteRule {
            source: "/reviews/:slug*".to_string(),
            destination: "/content/reviews/:slug*".to_string(),
        }
    );
    let view = ConfigView::new(&cfg);
    assert_eq!(view.nextjs_rewrites().len(), 2);
}

#[test]
fn storybook_config_parsed() {
    let cfg = load_v2_config(&fixture("with-storybook"), None).unwrap();
    assert!(matches!(
        &cfg.tests.playwright.configs,
        Some(StringOrList::Many(v)) if v.len() == 2
    ));
    assert!(cfg.tests.storybook.configs.is_some());
    assert!(cfg.tests.vitest.configs.is_some());
}

#[test]
fn test_plan_percent_values_accept_numbers_and_percent_strings() {
    assert_eq!(TestPlanPercent::Number(25.0).value(), Some(25.0));
    assert_eq!(
        TestPlanPercent::String(" 50% ".to_string()).value(),
        Some(50.0)
    );
    assert_eq!(TestPlanPercent::String("half".to_string()).value(), None);
}

#[test]
fn test_plan_global_config_fallback_parsed() {
    let cfg = load_v2_config(&fixture("test-plan-global-config-fallback"), None).unwrap();
    let environments = &cfg.test_plan.vitest.environments;

    assert_eq!(environments["camel"].global_config_fallback, Some(false));
    assert_eq!(environments["snake"].global_config_fallback, Some(true));
}

// ── legacy conversions ────────────────────────────────────────────────────────

#[test]
fn legacy_playwright_converted() {
    let cfg = load_v2_config(&fixture("legacy-playwright"), None).unwrap();
    assert_eq!(cfg.projects["web"].type_, Some(ProjectType::Nextjs));
    assert_eq!(cfg.projects["web"].root.as_deref(), Some("web/app"));
    let pw = &cfg.tests.playwright;
    assert!(matches!(&pw.configs, Some(StringOrList::One(s)) if s == "playwright.config.mts"));
    assert!(pw.selectors.test_ids.contains(&"data-pw".to_string()));
    assert_eq!(pw.selectors.component_test_ids["dataPw"], "data-pw");
    assert_eq!(pw.selector_roots, vec!["web/app", "web/components"]);
}

#[test]
fn legacy_guardrails_converted() {
    let cfg = load_v2_config(&fixture("legacy-guardrails"), None).unwrap();
    assert_eq!(cfg.projects["backend"].root.as_deref(), Some("backend"));
    assert_eq!(
        cfg.filesystem.skip_directories,
        vec![".next", "node_modules"]
    );
    assert!(cfg.rule_configured("http-route-static-paths"));
}

#[test]
fn legacy_react_traits_converted() {
    let cfg = load_v2_config(&fixture("legacy-react-traits"), None).unwrap();
    assert!(cfg.projects.contains_key("web"));
    assert_eq!(cfg.projects["web"].type_, Some(ProjectType::Nextjs));
    assert_eq!(cfg.projects["web"].root.as_deref(), Some("src/app"));
}

#[test]
fn legacy_next_to_fetch_converted() {
    let cfg = load_v2_config(&fixture("legacy-next-to-fetch"), None).unwrap();
    assert!(cfg.projects.contains_key("web"));
    assert_eq!(cfg.projects["web"].root.as_deref(), Some("app"));
}

// ── schema ────────────────────────────────────────────────────────────────────

#[test]
fn string_or_list_values_single() {
    let s = StringOrList::One("foo".to_string());
    assert_eq!(s.values(), vec!["foo"]);
}

#[test]
fn string_or_list_values_many() {
    let s = StringOrList::Many(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(s.values(), vec!["a", "b"]);
}

#[test]
fn rule_def_enabled_defaults_to_true() {
    let yaml = "{}";
    let def: RuleDef = serde_yaml::from_str(yaml).unwrap();
    assert!(def.enabled);
}

#[test]
fn project_and_rule_path_filters_parse() {
    let cfg = load_v2_config(&fixture("rule-path-filters"), None).unwrap();
    let project = &cfg.projects["web"];

    assert_eq!(project.include, vec!["src/**"]);
    assert_eq!(project.exclude, vec!["**/*.stories.tsx"]);
    assert_eq!(cfg.rules[0].include, vec!["**/*.ts"]);
    assert_eq!(cfg.rules[0].exclude, vec!["generated/**"]);
}

#[test]
fn invalid_rule_path_filter_errors() {
    let err = load_v2_config(&fixture("invalid-rule-path-filter"), None)
        .err()
        .unwrap();

    assert!(err.to_string().contains("rules[0].exclude"));
}

#[test]
fn v2_rule_applications_require_rule_id() {
    let err = load_v2_config(&fixture("missing-rule-id"), None)
        .err()
        .unwrap();

    assert!(err.to_string().contains("rules[0].rule is required"));
}

#[test]
fn rule_def_options_deserialized() {
    let cfg = load_v2_config(&fixture("multi-project"), None).unwrap();
    let rule = cfg.rule_applications("http-route-static-paths")[0];
    assert_eq!(
        rule.message.as_deref(),
        Some("Route paths must be static literals")
    );
    assert!(rule.enabled);

    #[derive(serde::Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Opts {
        backend_pattern: String,
    }
    let opts: Opts = rule.rule_options();
    assert_eq!(opts.backend_pattern, "backend/api/**");
}

#[test]
fn rule_def_options_returns_default_on_bad_type() {
    let rule = RuleDef::default();

    #[derive(serde::Deserialize, Default, PartialEq, Debug)]
    struct Opts {
        foo: String,
    }
    let opts: Opts = rule.rule_options();
    assert_eq!(opts, Opts::default());
}

#[test]
fn rule_application_options_return_all_effective_applications() {
    let cfg = load_v2_config(&fixture("multiple-rule-application-options"), None).unwrap();

    #[derive(serde::Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Opts {
        src_max: Option<usize>,
    }

    let opts = cfg.rule_application_options::<Opts>("rust-max-lines-per-file");

    assert_eq!(
        opts.iter().map(|opt| opt.src_max).collect::<Vec<_>>(),
        vec![Some(100), Some(80)]
    );
}

// ── detect_and_parse explicit tool paths ──────────────────────────────────────

#[test]
fn explicit_playwright_config_path_dispatched() {
    let dir = fixture("legacy-playwright");
    let explicit = dir.join(".playwright-ast-coverage.yaml");
    let cfg = load_v2_config(&dir, Some(&explicit)).unwrap();
    assert_eq!(cfg.projects["web"].type_, Some(ProjectType::Nextjs));
}

#[test]
fn explicit_react_traits_config_path_dispatched() {
    let dir = fixture("legacy-react-traits");
    let explicit = dir.join(".react-traits.yaml");
    let cfg = load_v2_config(&dir, Some(&explicit)).unwrap();
    assert_eq!(cfg.projects["web"].root.as_deref(), Some("src/app"));
}

#[test]
fn explicit_next_to_fetch_config_path_dispatched() {
    let dir = fixture("legacy-next-to-fetch");
    let explicit = dir.join(".next-to-fetch.yaml");
    let cfg = load_v2_config(&dir, Some(&explicit)).unwrap();
    assert_eq!(cfg.projects["web"].root.as_deref(), Some("app"));
}

// ── legacy defaults ───────────────────────────────────────────────────────────

#[test]
fn legacy_playwright_minimal_uses_defaults() {
    let cfg = load_v2_config(&fixture("legacy-playwright-minimal"), None).unwrap();
    assert_eq!(cfg.projects["web"].root.as_deref(), Some("app"));
    let pw = &cfg.tests.playwright;
    assert!(pw.selectors.test_ids.contains(&"data-testid".to_string()));
    assert!(pw.selectors.test_ids.contains(&"data-pw".to_string()));
    assert_eq!(pw.selector_roots, vec!["app"]);
}

#[test]
fn legacy_simple_no_frontend_root_returns_empty_projects() {
    let cfg = load_v2_config(&fixture("legacy-react-traits-minimal"), None).unwrap();
    assert!(cfg.projects.is_empty());
}

#[test]
fn legacy_guardrails_disabled_rule_converted() {
    let cfg = load_v2_config(&fixture("legacy-guardrails-disabled"), None).unwrap();
    let view = ConfigView::new(&cfg);
    let rules = view.enabled_rules_for("backend");
    assert!(rules.iter().any(|(id, _)| *id == "active-rule"));
    assert!(!rules.iter().any(|(id, _)| *id == "disabled-rule"));
}

#[test]
fn legacy_guardrails_project_rule_without_top_level_options_converted() {
    let cfg = load_v2_config(&fixture("legacy-guardrails-project-rule-only"), None).unwrap();
    assert!(cfg.rule_configured("unique-exports"));
    assert!(cfg
        .rule_applications("unique-exports")
        .iter()
        .any(|rule| rule.projects == vec!["app"]));
}

#[test]
fn config_view_rule_applications_are_project_scoped() {
    let cfg = load_v2_config(&fixture("project-unknown-rule"), None).unwrap();
    let view = ConfigView::new(&cfg);
    let rules = view.enabled_rules_for("backend");
    assert!(rules.iter().any(|(id, _)| *id == "known-rule"));
    assert!(rules.iter().any(|(id, _)| *id == "ghost-rule"));
}

// ── parse error propagation ───────────────────────────────────────────────────

#[test]
fn malformed_playwright_config_errors() {
    let dir = fixture("legacy-playwright-malformed");
    let explicit = dir.join(".playwright-ast-coverage.yaml");
    let err = load_v2_config(&dir, Some(&explicit)).err().unwrap();
    assert!(!err.to_string().is_empty());
}

#[test]
fn malformed_guardrails_config_errors() {
    let err = load_v2_config(&fixture("legacy-guardrails-malformed"), None)
        .err()
        .unwrap();
    assert!(!err.to_string().is_empty());
}

#[test]
fn malformed_react_traits_config_errors() {
    let err = load_v2_config(&fixture("legacy-react-traits-malformed"), None)
        .err()
        .unwrap();
    assert!(!err.to_string().is_empty());
}

// ── find_config_root ──────────────────────────────────────────────────────────

#[test]
fn find_config_root_v2_stem_returns_root() {
    let dir = fixture("basic");
    assert_eq!(find_config_root(&dir), dir);
}

#[test]
fn find_config_root_tool_stem_returns_root() {
    let dir = fixture("legacy-playwright");
    assert_eq!(find_config_root(&dir), dir);
}

#[test]
fn find_config_root_guardrails_returns_containing_dir() {
    let dir = fixture("legacy-guardrails");
    assert_eq!(find_config_root(&dir), dir);
}

#[test]
fn find_config_root_no_config_returns_start() {
    let dir = fixture("empty");
    assert_eq!(find_config_root(&dir), dir);
}
