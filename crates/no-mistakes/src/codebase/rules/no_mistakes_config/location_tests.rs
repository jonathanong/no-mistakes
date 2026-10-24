use super::tests::fixture;
use super::*;

fn run(name: &str, explicit: Option<&str>) -> Vec<RuleFinding> {
    let root = fixture(name);
    let config = crate::config::v2::load_v2_config(&root, explicit.map(Path::new)).unwrap();
    let path = root.join(explicit.unwrap_or(".no-mistakes.yml"));
    check_with_files(&root, &config, &[path]).unwrap()
}

fn actual(findings: &[RuleFinding]) -> Vec<(usize, &str)> {
    findings
        .iter()
        .map(|finding| {
            assert_eq!(finding.rule, RULE_ID);
            (finding.line, finding.message.as_str())
        })
        .collect()
}

#[test]
fn findings_identify_rules_and_distinct_authored_entries() {
    let findings = run("locations", None);
    assert!(findings
        .iter()
        .all(|finding| finding.file == ".no-mistakes.yml"));
    assert_eq!(actual(&findings), vec![
        (4, "projects.web.with[brackets].root: missing directory `missing-app`"),
        (11, "rules[1] (unique-exports).exclude[0]: glob `gone/**/*.ts` matches no tracked files"),
        (12, "rules[1] (unique-exports).exclude[1]: glob `gone/**/*.ts` matches no tracked files"),
        (17, "rules[2] (forbidden-dependencies).options.roots[0]: missing directory `missing-first.ts`"),
        // Non-string option values are filtered for display, not for source positions.
        (19, "rules[2] (forbidden-dependencies).options.roots[1]: missing directory `missing-last.ts`"),
    ]);
}

#[test]
fn aliases_flow_multiline_duplicates_and_normalized_shapes_keep_authored_lines() {
    assert_eq!(actual(&run("locations-shapes", None)), vec![
        (4, "projects.alias.root: missing directory `missing-alias`"),
        (5, "projects.flow.name[0].root: missing directory `missing-flow`"),
        (7, "projects.folded.root: missing directory `missing-folded`"),
        (10, "projects.duplicate.root: missing directory `missing-last`"),
        (13, "tests.vitest.configs[0]: missing file `missing.config.ts`"),
        (19, "testPlan.vitest.fullSuiteTriggers.triggers[0].paths[0]: missing file `missing-trigger.ts`"),
        (25, "rules[1] (unique-exports).exclude[0]: glob `gone/**/*.ts` matches no tracked files"),
        (26, "rules[1] (unique-exports).exclude[1]: glob `gone/**/*.ts` matches no tracked files"),
    ]);
    assert_eq!(
        actual(&run("locations-targeted", None)),
        vec![(
            8,
            "testPlan.vitest.fullSuiteTriggers.projects.app[0]: missing file `missing-target.ts`"
        ),]
    );
}

#[test]
fn explicit_loaded_path_wins_even_outside_the_filtered_inventory() {
    let root = fixture("locations-custom");
    let config = crate::config::v2::load_v2_config(&root, Some(Path::new("policy.yml"))).unwrap();
    let sources = super::super::source_store_for_files(&[]);
    let findings = check_with_files_and_sources(&root, &config, &[], &sources).unwrap();
    assert_eq!(findings[0].file, "policy.yml");
    assert_eq!(
        actual(&findings),
        vec![(
            7,
            "rules[1] (unique-exports).exclude[0]: glob `gone/**/*.ts` matches no tracked files"
        ),]
    );
    assert_eq!(sources.physical_read_count(), 1);
    check_with_files_and_sources(&root, &config, &[], &sources).unwrap();
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn caller_created_configs_do_not_guess_locations_from_neighboring_files() {
    let root = fixture("locations");
    let config: NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let findings = check_with_files(&root, &config, &[root.join(".no-mistakes.yml")]).unwrap();
    assert_eq!(findings.len(), 5);
    assert!(findings.iter().all(|finding| finding.line == 1));
    // Other presets retain their canonical field spelling without provenance allocation.
    let references = paths::references(&config);
    assert!(references
        .iter()
        .all(|reference| reference.origin.is_none()));
    assert!(references
        .iter()
        .any(|reference| reference.field == "rules[2].options.roots[1]"));
}

#[test]
fn tagged_keys_keep_findings_without_guessing_an_unrelated_source_line() {
    let findings = run("locations-tagged", None);
    assert_eq!(
        actual(&findings),
        vec![(
            1,
            "rules[1] (forbidden-dependencies).options.roots[0]: missing directory `missing-real.ts`"
        )]
    );
}

#[test]
fn disabled_config_rule_does_not_read_provenance() {
    let root = fixture("locations");
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    config.rules[0].enabled = false;
    let sources = super::super::source_store_for_files(&[]);
    assert!(check_with_files_and_sources(&root, &config, &[], &sources)
        .unwrap()
        .is_empty());
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn malformed_globs_keep_fatal_policy_with_exact_entry_and_rule_context() {
    for (name, line, field) in [
        (
            "locations-invalid-exclude",
            8,
            "rules[1] (unique-exports).exclude[1]",
        ),
        (
            "locations-invalid-include",
            11,
            "rules[2] (forbidden-dependencies).include[1]",
        ),
        (
            "locations-invalid-project",
            5,
            "projects.app.with[brackets].include[1]",
        ),
    ] {
        let root = fixture(name);
        let error = crate::config::v2::load_v2_config(&root, None)
            .unwrap_err()
            .to_string();
        assert_eq!(error, format!(
            "{}:{line}: {field}: contains invalid glob `[bad`: error parsing glob '[bad': unclosed character class; missing ']'",
            root.join(".no-mistakes.yml").display()));
    }
}

#[test]
fn deprecated_dependencies_keep_authored_locations_and_loader_precedence() {
    for (fixture, line, field, missing) in [
        (
            "locations-deprecated-projects",
            9,
            "testPlan.vitest.fullSuiteTriggers.projects.app[0]",
            "missing-deprecated-project.ts",
        ),
        (
            "locations-deprecated-list",
            7,
            "testPlan.vitest.fullSuiteTriggers.triggers[0].paths[0]",
            "missing-deprecated-list.ts",
        ),
        (
            "locations-deprecated-precedence",
            12,
            "testPlan.vitest.fullSuiteTriggers.projects.app[0]",
            "missing-canonical.ts",
        ),
        (
            "locations-deprecated-null",
            9,
            "testPlan.vitest.fullSuiteTriggers.triggers[0].paths[0]",
            "missing-null-fallback.ts",
        ),
    ] {
        let findings = run(fixture, None);
        assert_eq!(findings.len(), 1, "{fixture}: {findings:?}");
        let expected = format!("{field}: missing file `{missing}`");
        assert_eq!(
            actual(&findings),
            vec![(line, expected.as_str())],
            "{fixture}"
        );
    }
}
