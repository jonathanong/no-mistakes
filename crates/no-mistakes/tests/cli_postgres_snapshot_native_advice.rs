use super::check_json_snapshot::assert_saved_fixture;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/native-parse-advisories")
        .join(name)
}

#[test]
fn native_parse_advice_check_json_snapshot() {
    let mut cases = Vec::new();
    for (label, name) in [
        ("parse_only", "parse-only"),
        ("mixed", "mixed"),
        ("no_newline", "no-newline"),
        ("valid", "valid-only"),
        ("embedded", "embedded"),
        ("same_line", "same-line-failures"),
    ] {
        for policy in ["fail", "advise", "ignore"] {
            let root = fixture(name);
            cases.push((
                format!("{label}_{policy}"),
                root.clone(),
                root.join(format!("{policy}.json")),
            ));
        }
    }
    for mode in ["line-suppressed", "file-suppressed", "inline-suppressed"] {
        for rule in [
            "postgres-bounded-statements",
            "postgres-conflict-ordering",
            "postgres-explicit-columns",
            "postgres-required-predicates",
            "postgres-sql-shape-policy",
        ] {
            for policy in ["fail", "advise", "ignore"] {
                let root = fixture(mode).join(rule);
                cases.push((
                    format!("{mode}_{rule}_{policy}"),
                    root.clone(),
                    root.join(format!("{policy}.json")),
                ));
            }
        }
    }
    let root = fixture("applications");
    cases.push((
        "applications".to_string(),
        root.clone(),
        root.join("applications.json"),
    ));
    let borrowed = cases
        .iter()
        .map(|(label, root, config)| (label.as_str(), root.clone(), config.clone()))
        .collect::<Vec<_>>();
    assert_saved_fixture("postgres_native_parse_advice", &borrowed);
}
