use super::support::{check, copy_tree, fixtures, Database};

/// The rule reads a catalog generated from real PostgreSQL: its unique, partial and deferrable
/// indexes decide which statements are bounded.
#[test]
fn bounded_statements_reads_a_generated_catalog() {
    let Some(database) = Database::create("bounded") else {
        return;
    };
    let source = fixtures().join("bounded");
    database.load(&source.join("schema.sql"));
    let project = tempfile::tempdir().unwrap();
    copy_tree(&source, project.path());
    let output = database.generate(
        "bounded_demo",
        Some("complete"),
        &project.path().join("schema.json"),
    );
    assert!(output.status.success());
    let (_, findings) = check(project.path(), &project.path().join(".no-mistakes.yml"));
    let found: Vec<_> = findings
        .iter()
        .filter(|(rule, _, _)| rule == "postgres-bounded-statements")
        .map(|(_, file, message)| (file.as_str(), message.as_str()))
        .collect();
    let flagged = [
        (
            "sql/queries.sql:1:",
            "SELECT can return every row of invoices",
        ),
        (
            "sql/queries.sql:2:",
            "UPDATE can change every row of exports",
        ),
        (
            "sql/queries.sql:4:",
            "SELECT can return every row of orders",
        ),
        (
            "sql/queries.sql:5:",
            "SELECT can return every row of tokens",
        ),
    ];
    assert_eq!(found.len(), flagged.len(), "{found:#?}");
    for (location, text) in flagged {
        assert!(
            found
                .iter()
                .any(|(file, message)| *file == "sql/queries.sql"
                    && message.contains(location)
                    && message.contains(text)),
            "{location} {text}: {found:#?}"
        );
    }
}
