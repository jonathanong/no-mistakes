#[test]
fn schema_path_options_collect_only_declared_path_fields() {
    let config: NoMistakesConfig = serde_yaml::from_str(
        r#"
projects:
  app: { root: apps/app }
  root: { root: . }
  all: { root: apps/all }
testPlan:
  vitest:
    fullSuiteTriggers:
      projects:
        app: { paths: [src/**/*.ts, "!src/generated/**"], targets: [web] }
        root: [root/**/*.ts]
        all: true
      triggers:
        - { name: config, paths: [config/**/*.ts, "!config/generated/**"] }
rules:
  - rule: csharp-max-lines-per-file
    options: { roots: [src, 42], testRoots: tests/**/*.cs }
  - rule: doc-consistency
    options: { requiredFiles: [README.md, ""], requiredSubstrings: [{ file: docs/a.md }, { file: 42 }] }
  - rule: file-extension-policy
    options: { allowlist: allow.txt, scopes: [{ path: web }] }
  - rule: forbidden-dependencies
    options: { roots: src, forbiddenFiles: banned.ts }
  - rule: forbidden-workspace-closure
    options: { lockfile: pnpm-lock.yaml }
  - rule: nextjs-redirect-destinations
    options: { configPath: next.config.ts, appRoot: app }
  - rule: package-json-workspace-coverage
    options: { packageRoots: packages, allowlist: packages/a/package.json }
  - rule: pnpm-release-age-policy
    options: { workspaceYaml: pnpm-workspace.yaml, dependabotPath: .github/dependabot.yml, lockfilePath: pnpm-lock.yaml }
  - rule: shellcheck-runner
    options: { shellFiles: script.sh, shebangDirs: bin, skillsLockfile: skills.lock }
  - rule: strict-package-layout
    options: { packages: [{ root: packages/a }] }
  - rule: tsconfig-alias-folder-mapping
    options: { tsconfig: tsconfig.json, baseDir: src }
  - rule: tsconfig-file-coverage
    options: { allow: [{ path: src/a.ts }], auxiliaryConfigs: [{ path: tsconfig.test.json }] }
  - rule: doc-consistency
    options:
      bannedPatterns:
        - pattern: PIN
        - files: ["docs/**/*.md", ""]
        - files: docs/*.md
  - rule: doc-consistency
    options: { bannedPatterns: true }
"#,
    )
    .unwrap();
    let refs = super::paths::references(&config);
    let fields: Vec<_> = refs
        .iter()
        .map(|reference| reference.field.as_str())
        .collect();
    for field in [
        "rules[0].options.roots[0]",
        "rules[1].options.requiredSubstrings[0].file",
        "rules[5].options.appRoot[0]",
        "rules[7].options.dependabotPath[0]",
        "rules[10].options.baseDir[0]",
        "rules[11].options.auxiliaryConfigs[0].path",
        "rules[12].options.bannedPatterns[1].files[0]",
        "rules[12].options.bannedPatterns[2].files[0]",
    ] {
        assert!(fields.contains(&field), "missing {field}: {fields:?}");
    }
    assert!(
        !fields
            .iter()
            .any(|field| field.contains("bannedPatterns[0]")),
        "an entry without files is not a path: {fields:?}"
    );
}
