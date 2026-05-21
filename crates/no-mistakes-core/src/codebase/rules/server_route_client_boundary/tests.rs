use super::*;
use crate::config::v2::schema::{Project, ProjectType, RuleDef, RuleScope};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/server-route-client-boundary")
        .join(name)
}

fn fixture_options(name: &str) -> serde_yaml::Value {
    let source = std::fs::read_to_string(fixture(name)).unwrap();
    serde_yaml::from_str(&source).unwrap()
}

fn config() -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.projects.insert(
        "backend".to_string(),
        Project {
            type_: Some(ProjectType::Server),
            root: Some("backend".to_string()),
            routes: vec!["api/**".to_string()],
            ..Default::default()
        },
    );
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        projects: vec!["backend".to_string()],
        ..Default::default()
    });
    config
}

#[test]
fn passes_when_clients_are_outside_route_folder() {
    let findings = check(&fixture("pass"), &config()).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn reports_client_file_in_route_folder() {
    let findings = check(&fixture("fail"), &config()).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "backend/api/client.ts");
    assert_eq!(findings[0].line, 5);
}

#[test]
fn generic_rule_runner_checks_server_route_client_boundary() {
    let root = fixture("fail");
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join("no-mistakes.yml")), None)
            .unwrap();

    assert_eq!(findings.len(), 1);
}

#[test]
fn generic_rule_runner_handles_boundary_without_dynamic_import_rule() {
    let root = fixture("fail");
    let findings = crate::codebase::rules::run_check(
        &root,
        Some(&root.join("boundary-only.no-mistakes.yml")),
        None,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
}

#[test]
fn generic_fact_rule_runner_checks_server_route_client_boundary() {
    let root = fixture("fail");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            source: true,
            ..Default::default()
        },
    );
    let findings = crate::codebase::rules::run_check_with_facts(
        &root,
        Some(&root.join("no-mistakes.yml")),
        None,
        &facts,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
}

#[test]
fn reports_same_file_route_and_client_mix() {
    let findings = check(&fixture("same-file"), &config()).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "backend/api/users.ts");
}

#[test]
fn disable_file_comment_suppresses_finding() {
    let findings = check(&fixture("disabled"), &config()).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn excludes_do_not_match_substrings() {
    let mut config = config();
    config.rules[0].options = fixture_options("options/exclude-substring.yml");

    let findings = check(&fixture("fail"), &config).unwrap();

    assert_eq!(findings.len(), 1);
}

#[test]
fn excludes_match_path_prefixes() {
    let mut config = config();
    config.rules[0].options = fixture_options("options/exclude-prefix.yml");

    let findings = check(&fixture("fail"), &config).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn glob_excludes_match_client_files() {
    let mut config = config();
    config.rules[0].options = fixture_options("options/exclude-client-glob.yml");

    let findings = check(&fixture("fail"), &config).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn excludes_apply_before_route_directories_are_seeded() {
    let mut config = config();
    config.rules[0].options = fixture_options("options/exclude-route-file.yml");

    let findings = check(&fixture("fail"), &config).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn excludes_normalize_dot_relative_paths() {
    let mut config = config();
    config.rules[0].options = fixture_options("options/exclude-dot-relative-route-file.yml");

    let findings = check(&fixture("fail"), &config).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn no_route_directory_returns_no_findings() {
    let findings = check(&fixture("no-route"), &config()).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn file_path_returns_empty_when_route_globs_are_unconfigured() {
    let mut config = config();
    config.projects.get_mut("backend").unwrap().routes.clear();

    let findings = check(&fixture("fail"), &config).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn non_matching_rules_return_no_findings() {
    let mut config = config();
    config.rules[0].projects = vec!["other".to_string()];

    let findings = check(&fixture("fail"), &config).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn fact_path_returns_empty_when_route_globs_are_unconfigured() {
    let root = fixture("no-route");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            source: true,
            ..Default::default()
        },
    );

    let findings = check_with_facts(&root, &config(), &facts).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn fact_path_returns_empty_when_no_route_directory_matches() {
    let mut config = config();
    config.projects.get_mut("backend").unwrap().routes.clear();
    let root = fixture("fail");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            source: true,
            ..Default::default()
        },
    );

    let findings = check_with_facts(&root, &config, &facts).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn fact_path_errors_when_source_facts_are_missing() {
    let root = fixture("fail");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan::default(),
    );

    let err = check_with_facts(&root, &config(), &facts).unwrap_err();

    assert!(err.to_string().contains("requires source facts"));
}

#[test]
fn route_globs_can_include_project_root() {
    let mut config = config();
    config.projects.get_mut("backend").unwrap().root = Some(".".to_string());
    config.projects.get_mut("backend").unwrap().routes = vec!["backend/api/**".to_string()];

    let findings = check(&fixture("fail"), &config).unwrap();

    assert_eq!(findings.len(), 1);
}

#[test]
fn route_globs_normalize_dot_prefixes() {
    let mut config = config();
    config.projects.get_mut("backend").unwrap().root = Some("./backend".to_string());
    config.projects.get_mut("backend").unwrap().routes = vec!["./api/**".to_string()];

    let findings = check(&fixture("dot-globs"), &config).unwrap();

    assert_eq!(findings.len(), 1);
}

#[test]
fn route_globs_skip_invalid_patterns_without_disabling_valid_routes() {
    let mut config = config();
    config.projects.get_mut("backend").unwrap().routes =
        vec!["[".to_string(), "api/**".to_string()];

    let findings = check(&fixture("fail"), &config).unwrap();

    assert_eq!(findings.len(), 1);
}

#[test]
fn del_routes_mark_route_folders() {
    let findings = check(&fixture("del-route"), &config()).unwrap();

    assert_eq!(findings.len(), 1);
}

#[test]
fn mount_routes_mark_route_folders() {
    let findings = check(&fixture("mount-route"), &config()).unwrap();

    assert_eq!(findings.len(), 1);
}

#[test]
fn computed_routes_mark_route_folders() {
    let findings = check(&fixture("computed-route"), &config()).unwrap();

    assert_eq!(findings.len(), 1);
}

#[test]
fn middleware_mounts_do_not_mark_route_folders() {
    let findings = check(&fixture("middleware-only"), &config()).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn module_scope_shadows_are_predeclared_before_calls() {
    let findings = check(&fixture("module-shadow"), &config()).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 12);
}

#[test]
fn chained_routes_mark_route_folders() {
    let findings = check(&fixture("chained-route"), &config()).unwrap();

    assert_eq!(findings.len(), 1);
}

#[test]
fn non_server_static_calls_do_not_mark_route_folders() {
    let findings = check(&fixture("non-server-call"), &config()).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn client_identifier_aliases_inline_require_and_shadows_are_handled() {
    let findings = check(&fixture("client-aliases"), &config()).unwrap();

    assert_eq!(findings.len(), 55);
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        vec![
            60, 61, 62, 64, 65, 66, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 79, 81, 83, 85, 87, 89,
            91, 92, 93, 94, 95, 96, 97, 98, 100, 112, 114, 123, 126, 128, 130, 134, 136, 145, 148,
            156, 158, 161, 162, 163, 164, 172, 194, 233, 238, 245, 256, 315, 320,
        ]
    );
}

#[test]
fn route_calls_inside_variable_initializers_mark_route_folders() {
    let findings = check(&fixture("assigned-route"), &config()).unwrap();

    assert_eq!(findings.len(), 1);
}

#[test]
fn project_scoped_rules_ignore_other_projects() {
    let mut config = config();
    config.projects.insert(
        "frontend".to_string(),
        Project {
            type_: Some(ProjectType::Server),
            root: Some("frontend".to_string()),
            routes: vec!["api/**".to_string()],
            ..Default::default()
        },
    );

    let findings = check(&fixture("fail"), &config).unwrap();

    assert_eq!(findings.len(), 1);
}

#[test]
fn excludes_ignore_non_normal_path_components() {
    let matcher = paths::ExcludeMatcher::new(&["var".to_string()]);

    assert!(!matcher.is_match(Path::new("/missing-root"), Path::new("/tmp/client.ts")));
}

#[test]
fn invalid_typescript_is_ignored_by_ast_helpers() {
    let path = fixture("invalid").join("backend/api/broken.ts");
    let source = std::fs::read_to_string(&path).unwrap();

    assert!(!ast::has_server_like_route_call(&path, &source));
    assert!(ast::client_call_lines(&path, &source).is_empty());
}

#[test]
fn adversarial_client_shapes_are_detected_without_routes() {
    let path = fixture("adversarial").join("backend/api/mixed.ts");
    let source = std::fs::read_to_string(&path).unwrap();

    assert!(!ast::has_server_like_route_call(&path, &source));
    assert_eq!(
        ast::client_call_lines(&path, &source),
        vec![11, 13, 14, 17, 29, 36, 39]
    );
}

#[test]
fn ast_helpers_fallback_to_typescript_for_unknown_extensions() {
    let source_path = fixture("adversarial").join("backend/api/mixed.ts");
    let source = std::fs::read_to_string(&source_path).unwrap();
    let unknown_path = Path::new("mixed.unknown");

    assert!(!ast::has_server_like_route_call(unknown_path, &source));
    assert_eq!(
        ast::client_call_lines(unknown_path, &source),
        vec![11, 13, 14, 17, 29, 36, 39]
    );
}

#[test]
fn detects_commonjs_clients_and_fact_path() {
    let root = fixture("cjs");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            source: true,
            ..Default::default()
        },
    );
    let findings = check_with_facts(&root, &config(), &facts).unwrap();
    assert_eq!(findings.len(), 2);
}

#[test]
fn repository_scoped_rule_uses_all_server_projects() {
    let mut config = config();
    config.rules[0].projects.clear();
    config.rules[0].scope = Some(RuleScope::Repository);

    let findings = check(&fixture("fail"), &config).unwrap();

    assert_eq!(findings.len(), 1);
}
