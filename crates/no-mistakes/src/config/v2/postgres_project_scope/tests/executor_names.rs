use super::parse;

fn saved(name: &str) -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/config/postgres-executor-names")
            .join(name),
    )
    .unwrap()
}

#[test]
fn extras_extend_defaults_and_replacements_remain_replacements() {
    for (file, expected) in [
        ("extra.yml", vec!["query", "read", "run", "write"]),
        ("replace.yml", vec!["run"]),
        ("project-extra.yml", vec!["query", "read", "run", "write"]),
        ("project-replace.yml", vec!["run"]),
    ] {
        let config = parse(&saved(file)).unwrap();
        let profiles = crate::codebase::postgres::configured_embedded_sql_options(
            &config,
            &["postgres-require-query-annotation"],
        )
        .unwrap();
        assert_eq!(profiles[0].executor_names, expected, "{file}");
        let key = if file.contains("extra") {
            "extraExecutorNames"
        } else {
            "executorNames"
        };
        assert!(config.rules[0].options.get(key).is_some(), "{file}");
    }
}

#[test]
fn project_rejects_both_names_options_even_when_empty() {
    let error = parse(&saved("project-conflict.yml"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("projects.db.postgres.executors"), "{error}");
    assert!(error.contains("cannot both be set"), "{error}");
}

#[test]
fn project_policies_cannot_disagree_or_be_restated() {
    for (file, text) in [
        ("project-disagree-replace.yml", "disagree on executorNames"),
        (
            "project-disagree-extra.yml",
            "disagree on extraExecutorNames",
        ),
        (
            "project-restates-extra.yml",
            "options.extraExecutorNames restates",
        ),
    ] {
        let error = parse(&saved(file)).unwrap_err().to_string();
        assert!(error.contains(text), "{file}: {error}");
    }
}

#[test]
fn empty_extension_retains_defaults_without_an_import_module() {
    for (file, expected) in [
        ("empty-extra.yml", vec!["query", "read", "write"]),
        ("empty-replace.yml", vec![]),
        ("project-empty-extra.yml", vec!["query", "read", "write"]),
        ("project-empty-replace.yml", vec!["query", "read", "write"]),
    ] {
        let config = parse(&saved(file)).unwrap();
        let profiles = crate::codebase::postgres::configured_embedded_sql_options(
            &config,
            &["postgres-require-query-annotation"],
        )
        .unwrap();
        assert_eq!(profiles[0].executor_names, expected, "{file}");
    }
}

#[test]
fn exclusions_follow_selection_and_project_inheritance() {
    for (file, expected) in [
        ("exclude-extra.yml", vec!["query", "write"]),
        ("exclude-replace.yml", vec![]),
        ("project-exclude-extra.yml", vec!["query", "write"]),
        ("project-exclude-replace.yml", vec![]),
    ] {
        let config = parse(&saved(file)).unwrap();
        let profiles = crate::codebase::postgres::configured_embedded_sql_options(
            &config,
            &["postgres-require-query-annotation"],
        )
        .unwrap();
        assert_eq!(profiles[0].executor_names, expected, "{file}");
        assert!(
            config.rules[0]
                .options
                .get("excludeExecutorNames")
                .is_some(),
            "{file}"
        );
    }
}

#[test]
fn project_exclusions_cannot_disagree_or_be_restated() {
    for (file, text) in [
        (
            "project-disagree-exclude.yml",
            "disagree on excludeExecutorNames",
        ),
        (
            "project-restates-exclude.yml",
            "options.excludeExecutorNames restates",
        ),
    ] {
        let error = parse(&saved(file)).unwrap_err().to_string();
        assert!(error.contains(text), "{file}: {error}");
    }
}
