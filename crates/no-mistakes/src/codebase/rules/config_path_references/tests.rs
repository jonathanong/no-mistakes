use super::references::*;
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/config-path-references")
            .join(name),
    )
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
fn reports_missing_config_path_references() {
    let root = fixture_root("fixture");
    let files = vec![
        root.join("config/app.yml"),
        root.join("config/existing.json"),
        root.join("src/a.test.ts"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
files: [config/app.yml]
keys: [paths.required, paths.glob]
allowGlobs: true
"#,
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("missing.json"));
}

#[test]
fn glob_references_default_to_config_file_directory() {
    let root = fixture_root("fixture");
    let opts = Options {
        allow_globs: true,
        ..Default::default()
    };
    let rel_files = vec!["config/existing.json".to_string()];

    assert!(reference_exists(
        &root,
        &root.join("config/app.yml"),
        &opts,
        "*.json",
        &rel_files
    )
    .unwrap());
}

#[test]
fn glob_references_match_targets_outside_rule_include_filter() {
    let root = fixture_root("fixture");
    let mut config = config(
        r#"
files: [config/app.yml]
keys: [paths.glob]
allowGlobs: true
"#,
    );
    config.rules[0].include = vec!["config/app.yml".to_string()];
    let files = vec![
        root.join("config/app.yml"),
        root.join("config/existing.json"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn glob_reference_patterns_handle_root_base_and_root_level_configs() {
    let root = fixture_root("fixture");
    let root_opts = Options {
        allow_globs: true,
        base_dir: BaseDir::Root,
        ..Default::default()
    };
    let config_file_opts = Options {
        allow_globs: true,
        ..Default::default()
    };

    assert_eq!(
        reference_pattern(&root, &root.join("config/app.yml"), &root_opts, "src/*.ts"),
        "src/*.ts"
    );
    assert_eq!(
        reference_pattern(&root, &root.join("app.yml"), &config_file_opts, "*.json"),
        "*.json"
    );
    assert_eq!(
        reference_pattern(&root, Path::new(""), &config_file_opts, "*.json"),
        "*.json"
    );
}

#[test]
fn glob_reference_patterns_normalize_relative_segments_and_escape_config_dirs() {
    let root = fixture_root("fixture");
    let opts = Options {
        allow_globs: true,
        ..Default::default()
    };
    let rel_files = vec![
        "schemas/user.json".to_string(),
        "apps/[tenant]/foo.json".to_string(),
    ];

    assert!(reference_exists(
        &root,
        &root.join("config/app.yml"),
        &opts,
        "../schemas/*.json",
        &rel_files
    )
    .unwrap());
    assert_eq!(
        reference_pattern(
            &root,
            &root.join("config/app.yml"),
            &opts,
            "../../secrets/*.json"
        ),
        "../secrets/*.json"
    );
    assert_eq!(
        reference_pattern(
            &root,
            &root.join("apps/[tenant]/config.yml"),
            &opts,
            "*.json"
        ),
        r#"apps/\[tenant\]/*.json"#
    );
    assert!(reference_exists(
        &root,
        &root.join("apps/[tenant]/config.yml"),
        &opts,
        "*.json",
        &rel_files
    )
    .unwrap());
}

#[test]
fn invalid_glob_references_surface_errors() {
    let root = fixture_root("fixture");
    let opts = Options {
        allow_globs: true,
        ..Default::default()
    };

    assert!(reference_exists(&root, &root.join("config/app.yml"), &opts, "{", &[]).is_err());
}

#[test]
fn brace_alternates_are_treated_as_globs() {
    let root = fixture_root("fixture");
    let opts = Options {
        allow_globs: true,
        ..Default::default()
    };
    let rel_files = vec!["config/existing.json".to_string()];

    assert!(reference_exists(
        &root,
        &root.join("config/app.yml"),
        &opts,
        "{existing,missing}.json",
        &rel_files
    )
    .unwrap());
}

#[test]
fn existing_literal_paths_with_glob_metacharacters_win_before_glob_matching() {
    let root = fixture_root("fixture");
    let opts = Options {
        allow_globs: true,
        base_dir: BaseDir::Root,
        ..Default::default()
    };

    assert!(reference_exists(
        &root,
        &root.join("config/app.yml"),
        &opts,
        "schemas/[tenant].json",
        &[]
    )
    .unwrap());
}

#[test]
fn bracketed_references_are_literals_when_missing() {
    let root = fixture_root("fixture");
    let opts = Options {
        allow_globs: true,
        base_dir: BaseDir::Root,
        ..Default::default()
    };
    let rel_files = vec!["schemas/t.json".to_string()];

    assert!(!reference_exists(
        &root,
        &root.join("config/app.yml"),
        &opts,
        "schemas/[missing].json",
        &rel_files
    )
    .unwrap());
}

#[test]
fn escaped_glob_metacharacters_do_not_enable_glob_matching() {
    assert!(!has_glob_metachar(r"schemas/\*.json"));
    assert!(!has_glob_metachar(r"schemas/\?.json"));
    assert!(!has_glob_metachar(r"schemas/path\"));
    assert!(has_glob_metachar("schemas/{tenant,default}.json"));
    assert!(has_glob_metachar("schemas/?.json"));
}

#[test]
fn literal_references_outside_the_repository_do_not_exist_for_this_rule() {
    let root = fixture_root("fixture");
    let opts = Options::default();

    assert!(!reference_exists(
        &root,
        &root.join("config/app.yml"),
        &opts,
        "../../../../../Cargo.toml",
        &[]
    )
    .unwrap());
}

#[test]
fn ignores_unreadable_or_invalid_config_files() {
    let root = fixture_root("fixture");
    let files = vec![root.join("config"), root.join("config/invalid.yml")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
files: [config, config/invalid.yml]
keys: [paths.required]
"#,
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("failed to parse YAML"),
        "{findings:?}"
    );
}

#[test]
fn extracts_only_string_or_sequence_values_at_config_keys() {
    let value: serde_yaml::Value = serde_yaml::from_str(
        r#"
paths:
  sequence: [one.json, 2]
  object:
    nested: true
"#,
    )
    .unwrap();

    assert_eq!(values_at_key(&value, "paths.missing"), Vec::<String>::new());
    assert_eq!(
        values_at_key(&value, "paths.sequence"),
        vec!["one.json".to_string()]
    );
    assert_eq!(values_at_key(&value, "paths.object"), Vec::<String>::new());
}

#[test]
fn can_resolve_references_from_repository_root() {
    let root = fixture_root("fixture");
    let opts = Options {
        base_dir: BaseDir::Root,
        ..Default::default()
    };

    assert!(reference_exists(
        &root,
        &root.join("config/app.yml"),
        &opts,
        "config/existing.json",
        &[]
    )
    .unwrap());
}

fn preset_opts() -> String {
    r#"
presets: [oxlintrc, knip, dependabot, sgconfig, syncpack, coverage-rules, pnpm-workspace-filters]
"#
    .to_string()
}

fn listed(root: &Path, rels: &[&str]) -> Vec<PathBuf> {
    rels.iter().map(|rel| root.join(rel)).collect()
}

#[test]
fn presets_report_missing_required_paths() {
    let root = fixture_root("presets-fail");
    let files = listed(
        &root,
        &[
            ".oxlintrc.json",
            "packages/app/.oxlintrc.json",
            "knip.jsonc",
            ".github/dependabot.yml",
            "sgconfig.yml",
            ".syncpackrc.json",
            ".coverage-rules.yml",
            ".github/workflows/pnpm-filters.yml",
        ],
    );
    let findings = check_with_files(&root, &config(&preset_opts()), &files).unwrap();
    let messages: Vec<_> = findings
        .iter()
        .map(|finding| finding.message.clone())
        .collect();
    assert!(
        messages.iter().any(|m| m.contains("plugins/missing.js")),
        "{messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|m| m.contains("packages/app/.oxlintrc.json") && m.contains("nested-missing.js")),
        "{messages:?}"
    );
    assert!(
        messages.iter().any(|m| m.contains("src/missing.ts")),
        "{messages:?}"
    );
    assert!(
        messages.iter().any(|m| m.contains("src/root-missing.ts")),
        "{messages:?}"
    );
    assert!(
        messages.iter().any(|m| m.contains("missing-app")),
        "{messages:?}"
    );
    assert!(
        messages.iter().any(|m| m.contains("missing-rules")),
        "{messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|m| m.contains("missing/*/package.json")),
        "{messages:?}"
    );
    assert!(
        messages.iter().any(|m| m.contains("missing/**/*.rs")),
        "{messages:?}"
    );
    assert!(
        !messages
            .iter()
            .any(|m| m.contains("generated") || m.contains("node_modules")),
        "{messages:?}"
    );
}

#[test]
fn presets_pass_when_required_paths_exist() {
    let root = fixture_root("presets-pass");
    let files = listed(
        &root,
        &[
            ".oxlintrc.json",
            "plugins/local.js",
            "src/app.ts",
            "packages/app/.oxlintrc.json",
            "packages/app/plugin.js",
            "packages/app/src/index.ts",
            "packages/app/package.json",
            "knip.jsonc",
            ".github/dependabot.yml",
            "sgconfig.yml",
            "rules/keep.yml",
            ".syncpackrc.json",
            ".coverage-rules.yml",
            ".github/workflows/pnpm-filters.yml",
        ],
    );
    let findings = check_with_files(&root, &config(&preset_opts()), &files).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn unknown_presets_are_ignored() {
    let root = fixture_root("presets-fail");
    let files = listed(&root, &[".oxlintrc.json", "knip.jsonc"]);
    let findings = check_with_files(&root, &config("presets: [unknown]"), &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn invalid_preset_config_surfaces_parse_errors() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sgconfig.yml");
    std::fs::write(&path, ":\n  -").unwrap();
    let findings = check_with_files(dir.path(), &config("presets: [sgconfig]"), &[path]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("failed to parse YAML"),
        "{findings:?}"
    );
}

#[test]
fn extract_covers_optional_and_empty_preset_shapes() {
    let knip: serde_yaml::Value = serde_yaml::from_str(
        r#"
workspaces:
  ".":
    entry: ["src/index.ts", "!src/generated.ts", "**/node_modules/**"]
  1: { entry: skipped }
"#,
    )
    .unwrap();
    let knip_refs: Vec<_> = presets::extract("knip", &knip)
        .into_iter()
        .map(|extracted| extracted.value)
        .collect();
    assert_eq!(knip_refs, vec!["src/index.ts".to_string()]);

    let empty: serde_yaml::Value = serde_yaml::from_str("updates: []").unwrap();
    assert!(presets::extract("dependabot", &empty).is_empty());
    assert!(presets::extract("unknown", &empty).is_empty());
    assert!(presets::extract("coverage-rules", &empty).is_empty());
    assert!(presets::extract("knip", &empty).is_empty());

    let coverage: serde_yaml::Value = serde_yaml::from_str(
        r#"
rules:
  - paths: src/**/*.ts
  - paths: "!vendor/**"
  - paths: { nested: true }
"#,
    )
    .unwrap();
    let coverage_refs: Vec<_> = presets::extract("coverage-rules", &coverage)
        .into_iter()
        .map(|extracted| extracted.value)
        .collect();
    assert_eq!(coverage_refs, vec!["src/**/*.ts".to_string()]);

    assert!(presets::matches_preset(
        "oxlintrc",
        ".oxlintrc.json",
        "pkg/.oxlintrc.json"
    ));
    assert!(!presets::matches_preset(
        "dependabot",
        "dependabot.yml",
        "dependabot.yml"
    ));
}

#[test]
fn pnpm_workspace_filter_preset_handles_braces_wildcards_and_guards() {
    let extracted = presets::pnpm_workspace_filters(
        r#"
pnpm install --filter '{./packages/app}...' \
  --filter "./src/*..."
if [ -d "./optional" ]; then pnpm install --filter './optional...'; fi
test -f ./another/package.json && pnpm install --filter './another...'
"#,
    );
    let values: Vec<_> = extracted.into_iter().map(|item| item.value).collect();
    assert_eq!(values, vec!["./packages/app", "./src/*"]);
    assert!(presets::matches_preset(
        "pnpm-workspace-filters",
        "action.yml",
        ".github/actions/setup/action.yml"
    ));
}
