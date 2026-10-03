#[test]
fn literals_and_quoted_names_keep_case_and_whitespace() {
    let normalize = super::normalize;
    assert_eq!(normalize("'"), "'");
    assert_eq!(normalize("STATUS = 'idle'"), normalize("status = 'idle'"));
    assert_ne!(normalize("status = 'IDLE'"), normalize("status = 'idle'"));
    assert_ne!(
        normalize("\"Status\" = 'a  b'"),
        normalize("\"status\" = 'a b'")
    );
    assert_ne!(
        normalize("status = $$IDLE$$"),
        normalize("status = $$idle$$")
    );
}

#[test]
fn saved_escaped_predicates_keep_literal_and_identifier_boundaries() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/predicate-escaping/sql/001.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let options: serde_yaml::Value = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/predicate-escaping/options.yml"
    )))
    .unwrap();
    let raw = options["shapeOptions"]["keysetOnlySweep"]["nonSelectivePredicates"][0]
        .as_str()
        .unwrap();
    assert_eq!(super::normalize(raw), facts.sweeps[0].conjuncts[1].text);

    assert_ne!(
        facts.sweeps[0].conjuncts[1].text,
        facts.sweeps[1].conjuncts[1].text
    );
    assert_ne!(
        facts.sweeps[2].conjuncts[1].text,
        facts.sweeps[3].conjuncts[1].text
    );
}

#[test]
fn saved_escape_string_predicates_keep_token_boundaries() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/predicate-escaping/sql/001.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let options: serde_yaml::Value = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/predicate-escaping/options.yml"
    )))
    .unwrap();
    let raw = options["shapeOptions"]["keysetOnlySweep"]["nonSelectivePredicates"][3]
        .as_str()
        .unwrap();
    assert_eq!(super::normalize(raw), facts.sweeps[6].conjuncts[1].text);
    assert_ne!(
        facts.sweeps[6].conjuncts[1].text,
        facts.sweeps[7].conjuncts[1].text
    );
}

#[test]
fn saved_postgres_literal_spellings_preserve_values_and_boundaries() {
    let configured: serde_yaml::Value = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/predicate-escaping/options-normalization.yml"
    )))
    .unwrap();
    let configured = configured["shapeOptions"]["keysetOnlySweep"]["nonSelectivePredicates"]
        .as_sequence()
        .unwrap();
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/predicate-escaping/sql/002.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.sweeps.len(), 10);

    let normalized: Vec<String> = configured
        .iter()
        .map(|predicate| super::normalize(predicate.as_str().unwrap()))
        .collect();
    for (index, configured) in normalized.iter().enumerate() {
        assert_eq!(configured, &facts.sweeps[index * 2].conjuncts[1].text);
    }

    assert_ne!(normalized[0], facts.sweeps[1].conjuncts[1].text);
    assert_ne!(normalized[1], facts.sweeps[3].conjuncts[1].text);
    assert_ne!(normalized[2], facts.sweeps[5].conjuncts[1].text);
    assert_eq!(normalized[3], facts.sweeps[6].conjuncts[1].text);
    assert_ne!(normalized[3], facts.sweeps[7].conjuncts[1].text);
    assert_eq!(normalized[4], facts.sweeps[8].conjuncts[1].text);
    assert_ne!(normalized[4], facts.sweeps[9].conjuncts[1].text);
}
