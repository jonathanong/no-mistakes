use super::*;

#[test]
fn unknown_presets_are_rejected() {
    let root = fixture_root("presets-fail");
    let files = listed(&root, &[".oxlintrc.json", "knip.jsonc"]);
    let error = check_with_files(&root, &config("presets: [typo]"), &files).unwrap_err();
    assert!(
        error.to_string().contains("unsupported preset(s): typo"),
        "{error:#}"
    );
}

#[test]
fn no_mistakes_preset_only_extracts_rule_aware_path_options() {
    let config: NoMistakesConfig = serde_yaml::from_str(
        r#"
rules:
  - rule: workspace-package-cycles
    options:
      allowlist: ["@x/domain -> @x/api -> @x/domain"]
  - rule: package-json-workspace-coverage
    options:
      packageRoots: [packages]
      allowlist: [packages/allowed/package.json]
"#,
    )
    .unwrap();

    let extracted = crate::codebase::rules::no_mistakes_config::paths::references(&config);
    let values: Vec<_> = extracted
        .into_iter()
        .map(|reference| reference.value)
        .collect();
    assert_eq!(
        values,
        vec!["packages", "packages/allowed/package.json"],
        "workspace cycle identities are not paths"
    );
}

#[test]
fn no_mistakes_preset_extracts_documented_rule_option_paths() {
    let config: NoMistakesConfig = serde_yaml::from_str(
        r#"
rules:
  - rule: shellcheck-runner
    options:
      shellFiles: [scripts/check.sh]
      shebangDirs: [scripts]
      skillsLockfile: skills.lock
  - rule: package-json-workspace-coverage
    options:
      packageRoots: [packages]
  - rule: pnpm-release-age-policy
    options:
      workspaceYaml: pnpm-workspace.yaml
      dependabotPath: .github/dependabot.yml
      lockfilePath: pnpm-lock.yaml
  - rule: tsconfig-file-coverage
    options:
      allow: [{ path: scripts/generate.ts, reason: generated }]
      auxiliaryConfigs: [{ path: tsconfig.tools.json, reason: tools }]
  - rule: strict-package-layout
    options:
      packages: [{ root: packages }]
"#,
    )
    .unwrap();

    let extracted = crate::codebase::rules::no_mistakes_config::paths::references(&config);
    let values: Vec<_> = extracted
        .into_iter()
        .map(|reference| reference.value)
        .collect();
    assert_eq!(
        values,
        vec![
            "scripts/check.sh",
            "scripts",
            "skills.lock",
            "packages",
            "pnpm-workspace.yaml",
            ".github/dependabot.yml",
            "pnpm-lock.yaml",
            "scripts/generate.ts",
            "tsconfig.tools.json",
            "packages",
        ]
    );
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
fn preset_scanning_surfaces_parse_and_glob_errors() {
    let dir = tempfile::tempdir().unwrap();
    let workflow = dir.path().join(".github/workflows/check.yml");
    std::fs::create_dir_all(workflow.parent().unwrap()).unwrap();
    std::fs::write(&workflow, ":\n  -").unwrap();
    let findings = check_with_files(
        dir.path(),
        &config("presets: [pnpm-workspace-filters]"),
        std::slice::from_ref(&workflow),
    )
    .unwrap();
    assert!(findings[0].message.contains("failed to parse YAML"));

    std::fs::write(
        &workflow,
        "jobs:\n  check:\n    steps:\n      - run: pnpm install --filter './{'\n",
    )
    .unwrap();
    assert!(check_with_files(
        dir.path(),
        &config("presets: [pnpm-workspace-filters]"),
        std::slice::from_ref(&workflow),
    )
    .is_err());

    let syncpack = dir.path().join(".syncpackrc.json");
    std::fs::write(&syncpack, r#"{"source":["{"]}"#).unwrap();
    assert!(check_with_files(
        dir.path(),
        &config("presets: [syncpack]"),
        std::slice::from_ref(&syncpack),
    )
    .is_err());

    let no_mistakes_path = dir.path().join(".no-mistakes.yml");
    std::fs::write(&no_mistakes_path, "version: 2\n").unwrap();
    let mut no_mistakes_config = config("presets: [no-mistakes]");
    no_mistakes_config.rules.push(RuleDef {
        rule: "package-json-workspace-coverage".to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("packageRoots: ['{']").unwrap(),
        ..Default::default()
    });
    assert!(check_with_files(
        dir.path(),
        &no_mistakes_config,
        std::slice::from_ref(&no_mistakes_path),
    )
    .is_err());
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
    assert!(presets::matches_preset(
        "no-mistakes",
        ".no-mistakes.yml",
        ".no-mistakes.yml"
    ));
    assert!(!presets::matches_preset(
        "no-mistakes",
        ".no-mistakes.yml",
        "packages/app/.no-mistakes.yml"
    ));
}
