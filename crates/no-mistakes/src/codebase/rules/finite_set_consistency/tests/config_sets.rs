use super::extract::{
    extract_markdown_table_code_cells, extract_sql_enum, extract_ts_array_literal,
    extract_ts_const_array_property, extract_yaml_sequence, ExtractedSet,
};
use super::*;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::{
    schema::{Project, RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[path = "call_literals_regressions.rs"]
mod call_literals_regressions;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/finite-set-consistency")
            .join(name),
    )
}

fn call_literal_fixture_root(case: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency/call-literals")
            .join(case),
    )
}

fn check_call_literal_fixture(case: &str, target: &str) -> anyhow::Result<Vec<RuleFinding>> {
    let root = call_literal_fixture_root(case);
    let files = vec![root.join("schedules.mts"), root.join("registry.mts")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config = call_literal_config(target);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        required_call_site_fact_files(&root, &config),
        crate::codebase::check_facts::CheckFactPlan {
            graph: crate::codebase::ts_source::facts::TsFactPlan {
                call_sites: true,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    check_with_files_sources_and_facts(&root, &config, &files, &sources, Some(&facts))
}

fn call_literal_config(target: &str) -> NoMistakesConfig {
    config(&format!(
        r#"
sets:
  - name: schedulerIds
    file: schedules.mts
    kind: ts-call-first-string-argument
    target: "{target}"
  - name: registryIds
    file: registry.mts
    kind: ts-const-array-property
    target: AI_AGENTS_SCHEDULED_JOBS
    property: id
comparisons:
  - left: schedulerIds
    right: registryIds
"#
    ))
}

fn config(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

#[test]
fn extracts_yaml_ts_array_and_markdown_sets() {
    let root = fixture_root("fixture");
    let types = std::fs::read_to_string(root.join("src/types.ts")).unwrap();
    let workspace = std::fs::read_to_string(root.join("pnpm-workspace.yaml")).unwrap();
    let docs = std::fs::read_to_string(root.join("docs/dependency-updates.md")).unwrap();

    assert_eq!(
        extract_yaml_sequence(&workspace, "minimumReleaseAgeExclude"),
        BTreeSet::from([
            "@acme/api".to_string(),
            "@acme/cli".to_string(),
            "@acme/web".to_string()
        ])
    );
    assert_eq!(
        extract_ts_const_array_property(&types, "FIRST_PARTY_EXEMPTIONS", "name"),
        BTreeSet::from([
            "@acme/api".to_string(),
            "@acme/docs".to_string(),
            "@acme/web".to_string()
        ])
    );
    assert_eq!(
        extract_ts_array_literal(&types, "FIRST_PARTY_NAMES"),
        BTreeSet::from(["@acme/api".to_string(), "@acme/web".to_string()])
    );
    assert_eq!(
        extract_markdown_table_code_cells(&docs),
        BTreeSet::from(["@acme/api".to_string(), "@acme/web".to_string()])
    );
}

#[test]
fn call_first_string_arguments_match_registry_ids() {
    let findings = check_call_literal_fixture("valid", "ai_agents.upsertJobScheduler").unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn prepared_call_facts_parse_each_source_once() {
    let root = call_literal_fixture_root("prepared-once");
    crate::ast::begin_parse_count(&root);
    let findings =
        check_call_literal_fixture("prepared-once", "ai_agents.upsertJobScheduler").unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
    assert_eq!(counts.get(&root.join("schedules.mts")), Some(&1));
    assert!(
        !counts.contains_key(&root.join("registry.mts")),
        "{counts:?}"
    );
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
}

#[test]
fn standalone_dispatcher_prepares_only_call_source_once() {
    let root = call_literal_fixture_root("standalone-once");
    let files = vec![root.join("schedules.mts"), root.join("registry.mts")];
    crate::ast::begin_parse_count(&root);
    let findings = crate::codebase::rules::filesystem_dispatch::run_filesystem_rules_with_config(
        &root,
        &call_literal_config("ai_agents.upsertJobScheduler"),
        &files,
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
    assert_eq!(counts.get(&root.join("schedules.mts")), Some(&1));
    assert_eq!(counts.len(), 1, "{counts:?}");
}

#[test]
fn call_fact_demand_resolves_each_configured_project_once() {
    let root = call_literal_fixture_root("project-scope");
    let mut config = call_literal_config("ai_agents.upsertJobScheduler");
    config.projects.insert(
        "app-a".to_string(),
        Project {
            root: Some("packages/app-a".to_string()),
            ..Default::default()
        },
    );
    config.projects.insert(
        "app-b".to_string(),
        Project {
            root: Some("packages/app-b".to_string()),
            ..Default::default()
        },
    );
    config.rules[0].scope = None;
    config.rules[0].projects = vec!["app-a".to_string(), "app-b".to_string()];

    assert_eq!(
        required_call_site_fact_files(&root, &config),
        vec![
            root.join("packages/app-a/schedules.mts"),
            root.join("packages/app-b/schedules.mts"),
        ]
    );
}

#[test]
fn call_first_string_arguments_catch_missing_scheduler_registry_entries() {
    let findings =
        check_call_literal_fixture("missing-registry", "ai_agents.upsertJobScheduler").unwrap();

    assert_eq!(findings.len(), 1, "{findings:?}");
    let finding = &findings[0];
    assert_eq!(finding.rule, RULE_ID);
    assert!(
        finding
            .message
            .contains("schedulerIds contains `reconcileRuntimeGenerations`")
            && finding.message.contains("registryIds does not"),
        "{finding:?}"
    );
}

#[test]
fn call_first_string_arguments_include_calls_on_local_member_receivers() {
    let findings =
        check_call_literal_fixture("local-receiver", "ai_agents.upsertJobScheduler").unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn call_first_string_arguments_exclude_synthetic_method_edges() {
    let findings = check_call_literal_fixture("synthetic-method", "register").unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn call_first_string_arguments_exclude_optional_chain_calls() {
    let findings =
        check_call_literal_fixture("optional-chain", "ai_agents.upsertJobScheduler").unwrap();

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0]
        .message
        .contains("found no calls matching target 'ai_agents.upsertJobScheduler'"));
}

#[test]
fn call_first_string_arguments_normalize_escapes_like_registry_literals() {
    let findings =
        check_call_literal_fixture("escaped-literals", "ai_agents.upsertJobScheduler").unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn call_first_string_arguments_fail_closed_for_non_literal_arguments() {
    let findings =
        check_call_literal_fixture("non-literal", "ai_agents.upsertJobScheduler").unwrap();

    assert_eq!(findings.len(), 1, "{findings:?}");
    let finding = &findings[0];
    assert_eq!(finding.rule, RULE_ID);
    assert!(
        finding.message.contains(
            "finite set 'schedulerIds' requires every 'ai_agents.upsertJobScheduler' call to have a static first string argument"
        ),
        "{finding:?}"
    );
    assert_eq!(
        finding.line, 8,
        "the finding must point at the dynamic call"
    );
}

#[test]
fn call_first_string_arguments_reject_missing_spread_and_interpolated_arguments() {
    let findings =
        check_call_literal_fixture("dynamic-arguments", "ai_agents.upsertJobScheduler").unwrap();

    assert_eq!(findings.len(), 3, "{findings:?}");
    assert!(findings.iter().all(|finding| {
        finding.rule == RULE_ID
            && finding.message.contains(
                "finite set 'schedulerIds' requires every 'ai_agents.upsertJobScheduler' call to have a static first string argument"
            )
    }));
}

#[test]
fn call_first_string_arguments_fail_closed_for_unknown_targets() {
    let findings = check_call_literal_fixture("wrong-target", "ai_agents.scheduleJob").unwrap();

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].rule, RULE_ID);
    assert!(findings[0]
        .message
        .contains("found no calls matching target 'ai_agents.scheduleJob'"));
}

#[test]
fn call_first_string_arguments_require_a_target() {
    let findings = check_call_literal_fixture("valid", "").unwrap();

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].rule, RULE_ID);
    assert!(findings[0]
        .message
        .contains("finite set 'schedulerIds' requires a non-empty target"));
}

#[test]
fn call_first_string_arguments_require_prepared_facts() {
    let root = call_literal_fixture_root("valid");
    let files = vec![root.join("schedules.mts"), root.join("registry.mts")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let findings = check_with_files_and_sources(
        &root,
        &call_literal_config("ai_agents.upsertJobScheduler"),
        &files,
        &sources,
    )
    .unwrap();

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0]
        .message
        .contains("has no prepared TypeScript facts"));
}

#[test]
fn call_first_string_arguments_report_prepared_parse_errors() {
    let findings =
        check_call_literal_fixture("parse-error", "ai_agents.upsertJobScheduler").unwrap();

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0]
        .message
        .contains("configured file failed to parse"));
}

#[test]
fn compares_yaml_ts_glob_and_markdown_sets() {
    let root = fixture_root("fixture");
    let files = vec![
        root.join("pnpm-workspace.yaml"),
        root.join("src/types.ts"),
        root.join(".github/dependabot.yml"),
        root.join("docs/dependency-updates.md"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
sets:
  - name: workspaceExcludes
    file: pnpm-workspace.yaml
    kind: yaml-sequence
    key: minimumReleaseAgeExclude
  - name: registry
    file: src/types.ts
    kind: ts-const-array-property
    target: FIRST_PARTY_EXEMPTIONS
    property: name
  - name: dependabotGlobs
    file: .github/dependabot.yml
    kind: yaml-sequence
    key: updates.0.cooldown.exclude
  - name: docsMentions
    file: docs/dependency-updates.md
    kind: markdown-table-code-cells
comparisons:
  - left: workspaceExcludes
    right: registry
  - left: registry
    right: dependabotGlobs
    mode: glob-coverage
  - left: registry
    right: docsMentions
    mode: mention
"#,
        ),
        &files,
    )
    .unwrap();
    let body = format!("{findings:?}");

    assert_eq!(findings.len(), 4, "{body}");
    assert!(
        body.contains("workspaceExcludes contains `@acme/cli`"),
        "{body}"
    );
    assert!(body.contains("registry contains `@acme/docs`"), "{body}");
    assert!(
        body.contains("no glob in dependabotGlobs covers it"),
        "{body}"
    );
    assert!(body.contains("docsMentions does not mention it"), "{body}");
}

#[test]
fn glob_coverage_reports_invalid_globs() {
    let root = fixture_root("fixture");
    let files = vec![root.join("src/types.ts"), root.join("pnpm-workspace.yaml")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
sets:
  - name: names
    file: src/types.ts
    kind: ts-array-literal
    target: FIRST_PARTY_NAMES
  - name: globs
    file: pnpm-workspace.yaml
    kind: yaml-sequence
    key: invalidGlobs
comparisons:
  - left: names
    right: globs
    mode: glob-coverage
"#,
        ),
        &files,
    )
    .unwrap();

    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("invalid glob")),
        "unexpected findings: {findings:?}"
    );
}

#[test]
fn config_set_extractors_cover_edge_cases() {
    assert!(extract_yaml_sequence(":", "packages").is_empty());
    assert!(extract_yaml_sequence("packages: nope", "packages").is_empty());
    assert!(extract_yaml_sequence("updates: []", "updates.0.exclude").is_empty());
    assert_eq!(
        extract_yaml_sequence("packages:\n  - one\n  - 2\n", "packages"),
        BTreeSet::from(["one".to_string()])
    );

    assert!(extract_ts_array_literal("const OTHER = ['a'];", "NAMES").is_empty());
    assert_eq!(
        extract_ts_array_literal(
            r#"const NAMES = ["a\"b", `template`, notString, 'single'];"#,
            "NAMES"
        ),
        BTreeSet::from([
            "a\"b".to_string(),
            "single".to_string(),
            "template".to_string()
        ])
    );
    assert!(extract_ts_array_literal(
        r#"const NAMES = getNames(); const OTHER = ["api"];"#,
        "NAMES"
    )
    .is_empty());
    assert_eq!(
        extract_ts_array_literal(
            r#"const NAMES = [
  // keep pinned
  "@acme/api",
  /*
   * keep local
   */
  "@acme/web",
];"#,
            "NAMES"
        ),
        BTreeSet::from(["@acme/api".to_string(), "@acme/web".to_string()])
    );
    assert!(extract_ts_array_literal(r#"const NAMES = ["unterminated];"#, "NAMES").is_empty());
    assert!(
        extract_ts_const_array_property("const OTHER = [{ name: 'api' }];", "ITEMS", "name")
            .is_empty()
    );
    assert!(extract_ts_const_array_property(
        r#"const ITEMS = [{ name: "unterminated }];"#,
        "ITEMS",
        "name"
    )
    .is_empty());
    assert!(extract_ts_const_array_property(
        r#"const ITEMS = getItems(); const OTHER = [{ name: "api" }];"#,
        "ITEMS",
        "name"
    )
    .is_empty());
    assert_eq!(
        extract_ts_const_array_property(
            r#"const ITEMS = [
  // first-party package
  { name: "api" },
  "ignored",
  /* documented in policy table */
  { name: `web`, other: "x" },
];"#,
            "ITEMS",
            "name"
        ),
        BTreeSet::from(["api".to_string(), "web".to_string()])
    );
    assert!(extract_yaml_sequence("packages:\n  nested: []\n", "packages.0").is_empty());
    assert!(extract_sql_enum("CREATE TYPE status AS ENUM ('open'", "status").is_empty());
}

#[test]
fn comparison_modes_cover_defaults_custom_messages_and_unknown_modes() {
    let left = ExtractedSet {
        file: "left.ts".to_string(),
        values: BTreeSet::from(["api".to_string(), "web".to_string()]),
        issues: Vec::new(),
    };
    let right = ExtractedSet {
        file: "right.md".to_string(),
        values: BTreeSet::from(["api".to_string()]),
        issues: Vec::new(),
    };

    let mut findings = Vec::new();
    super::comparison::compare(
        &left,
        &right,
        &Comparison {
            left: "left".to_string(),
            right: "right".to_string(),
            message: Some("sets differ".to_string()),
            ..Default::default()
        },
        &mut findings,
    );
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].message, "sets differ");

    findings.clear();
    super::comparison::compare(
        &left,
        &right,
        &Comparison {
            left: "left".to_string(),
            right: "right".to_string(),
            mode: "unknown".to_string(),
            ..Default::default()
        },
        &mut findings,
    );
    assert!(findings.is_empty());
}
