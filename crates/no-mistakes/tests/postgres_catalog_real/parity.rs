use super::support::{check, fixtures, Database};

/// Conflict and lock ordering must read the same facts from either coverage.
#[test]
fn ordering_findings_are_identical_for_complete_and_ordering_catalogs() {
    let Some(database) = Database::create("parity") else {
        return;
    };
    database.load(&fixtures().join("setup.sql"));
    let mut outcomes = Vec::new();
    for coverage in [Some("ordering"), None] {
        let project = tempfile::tempdir().unwrap();
        std::fs::copy(
            fixtures().join("parity.yml"),
            project.path().join(".no-mistakes.yml"),
        )
        .unwrap();
        for source in ["conflict-pass", "conflict-fail", "lock-pass", "lock-fail"] {
            let name = format!("{source}.ts");
            std::fs::copy(fixtures().join(&name), project.path().join(name)).unwrap();
        }
        let output = database.generate(
            "Catalog.Test",
            coverage,
            &project.path().join("catalog.json"),
        );
        assert!(output.status.success());
        let (_, findings) = check(project.path(), &project.path().join(".no-mistakes.yml"));
        outcomes.push(findings);
    }
    assert_eq!(
        outcomes[0], outcomes[1],
        "ordering and complete catalogs must produce identical findings"
    );
    // Not vacuous: each failing source is reported once and each passing source is clean.
    let reported = |name: &str| {
        outcomes[0]
            .iter()
            .filter(|(_, file, _)| file == name)
            .count()
    };
    assert_eq!(outcomes[0].len(), 2, "{:#?}", outcomes[0]);
    assert_eq!(reported("conflict-fail.ts"), 1);
    assert_eq!(reported("lock-fail.ts"), 1);
}
