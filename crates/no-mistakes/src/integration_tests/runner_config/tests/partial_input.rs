use super::*;
use crate::codebase::check_outcome::CheckInputFailure;

fn assert_input(error: anyhow::Error, file: &Path) {
    let input = error.downcast_ref::<CheckInputFailure>().unwrap();
    assert_eq!(input.file.as_deref(), Some(file));
    assert_eq!(input.property.as_deref(), Some("test.include"));
    assert!(input.message.contains("broken.config.ts:3: test.include"));
}

#[test]
fn runner_facts_and_vitest_catalog_preserve_imported_property_failures() {
    let fixture = saved_config_fixture("vitest-include-location");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let prepared = prepare_vitest(&root, StringOrList::One("vitest.config.ts".into()));
    let parsed = prepared.parse_all().unwrap();
    let file = root.join("broken.config.ts");
    assert_input(
        parsed
            .projects_for(&prepared, Framework::Vitest)
            .unwrap_err(),
        &file,
    );
    let facts = parsed.files.get(&root.join("vitest.config.ts")).unwrap();
    assert_input(facts.vitest_projects().unwrap().unwrap_err().into(), &file);

    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("vitest.config.ts".into()));
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible = snapshot.paths_for(&root);
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, &visible).unwrap();
    let catalog = crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig, None);
    let projects =
        crate::codebase::rules::prepare_vitest_project_catalog(&root, &config, &snapshot, &catalog);
    assert_input(projects.config_projects().unwrap_err(), &file);
}

#[test]
fn retained_input_failure_keeps_context_and_untyped_error_text() {
    let input = CheckInputFailure {
        file: Some(PathBuf::from("broken.config.ts")),
        property: Some("test.include".into()),
        message: "unsupported expression".into(),
    };
    let retained =
        CheckInputFailure::from_error(anyhow::Error::from(input.clone()).context("runner"));
    assert_eq!(retained.file, input.file);
    assert_eq!(retained.property, input.property);
    assert_eq!(retained.message, "runner: unsupported expression");
    let plain = CheckInputFailure::from_error(anyhow::anyhow!("untyped"));
    assert_eq!(plain.file, None);
    assert_eq!(plain.property, None);
    assert_eq!(plain.message, "untyped");
}

#[test]
fn prepared_runner_records_cached_parse_errors_as_project_results() {
    let root = fixture_root("parse-errors");
    let prepared = prepare_vitest(
        &root,
        StringOrList::One("vitest.syntax-error.mts".to_string()),
    );
    let path = root.join("vitest.syntax-error.mts");

    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let facts = prepared
        .parse_path_for_facts_with_session(&session, &path)
        .unwrap();
    assert_eq!(facts.results.len(), 1);
    assert!(facts.results[0]
        .projects
        .as_ref()
        .unwrap_err()
        .to_string()
        .contains("failed to parse"));
    assert!(prepared
        .parse_error(&root.join("unprepared.mts"), "ignored".to_string())
        .is_none());
}
