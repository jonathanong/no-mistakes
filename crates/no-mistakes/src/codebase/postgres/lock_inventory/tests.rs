use super::super::{extract_sql_statement_facts, statements::extract_sql_conflict_policy_facts};

fn fixture(name: &str) -> String {
    let path = crate::test_support::fixture_root("postgres-lock-inventory").join(name);
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn records_each_physical_lock_at_its_source_token() {
    let sql = fixture("locks.sql");
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .lock_inventory
        .iter()
        .map(|fact| (fact.category.as_str(), fact.line, fact.column))
        .collect();
    assert_eq!(
        sites,
        [
            ("for-update", 1, 21),
            ("for-no-key-update", 2, 21),
            ("nowait", 2, 54),
            ("for-share", 3, 21),
            ("skip-locked", 3, 31),
            ("for-key-share", 4, 21),
            ("for-update", 5, 52),
            ("nowait", 5, 71),
            ("for-share", 5, 78),
            ("for-update", 7, 25),
            ("for-share", 9, 26),
            ("lock-table", 10, 1),
            ("nowait", 10, 62),
            ("lock-table", 11, 1),
            ("advisory-lock", 12, 8),
            ("advisory-lock", 12, 29),
            ("advisory-lock", 13, 8),
            ("advisory-lock", 13, 58),
            ("for-update", 19, 37),
        ]
    );
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.name_parts == ["project_lock"]));
    let narrow = extract_sql_conflict_policy_facts(&sql, &[]);
    assert!(narrow.lock_inventory.is_empty());
}

#[test]
fn normalizes_trivia_but_retains_category_identifier_literal_and_multiplicity() {
    let sql = fixture("fingerprints.sql");
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    let inventory = facts.lock_inventory;
    assert_eq!(inventory.len(), 9);
    let fingerprints: Vec<_> = inventory.iter().map(|fact| &fact.fingerprint).collect();
    assert_eq!(fingerprints[0], fingerprints[1]);
    for variant in &fingerprints[2..5] {
        assert_ne!(fingerprints[0], *variant);
    }
    assert!(fingerprints[5..]
        .iter()
        .all(|fingerprint| *fingerprint == fingerprints[5]));
    let physical_sites: Vec<_> = inventory[5..]
        .iter()
        .map(|fact| (fact.line, fact.column))
        .collect();
    assert_eq!(physical_sites, [(8, 8), (8, 33), (9, 8), (9, 33)]);
}

#[test]
fn restores_procedural_strengths_and_multi_target_clauses_from_source() {
    let sql = fixture("procedural-strength.sql");
    let facts = extract_sql_statement_facts(&sql);
    let inventory = facts.lock_inventory;
    let sites: Vec<_> = inventory
        .iter()
        .map(|fact| (fact.category.as_str(), fact.line, fact.column))
        .collect();
    assert_eq!(
        sites,
        [
            ("for-no-key-update", 6, 42),
            ("nowait", 6, 84),
            ("for-key-share", 7, 42),
            ("skip-locked", 8, 15),
            ("for-no-key-update", 10, 9),
            ("nowait", 10, 55),
            ("for-update", 11, 42),
            ("for-share", 12, 42),
            ("for-no-key-update", 15, 38),
            ("nowait", 15, 56),
            ("for-key-share", 16, 38),
            ("skip-locked", 16, 52),
            ("for-no-key-update", 18, 1),
            ("nowait", 18, 27),
        ]
    );
    for (recovered, direct) in inventory[..6].iter().zip(&inventory[8..]) {
        assert_eq!(recovered.fingerprint, direct.fingerprint);
    }
}

#[test]
fn pairs_nested_and_parenthesized_queries_with_their_own_lock_clauses() {
    let sql = fixture("nested.sql");
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .lock_inventory
        .iter()
        .map(|fact| (fact.category.as_str(), fact.line, fact.column))
        .collect();
    assert_eq!(
        sites,
        [
            ("for-key-share", 3, 1),
            ("nowait", 3, 27),
            ("for-no-key-update", 4, 23),
            ("for-share", 5, 62),
            ("for-key-share", 6, 37),
            ("for-update", 6, 60),
        ]
    );
}

#[test]
fn locates_recovered_table_and_advisory_locks_at_the_original_source() {
    let sql = fixture("procedural-calls.sql");
    let facts = extract_sql_statement_facts(&sql);
    let sites: Vec<_> = facts
        .lock_inventory
        .iter()
        .map(|fact| (fact.category.as_str(), fact.line, fact.column))
        .collect();
    assert_eq!(
        sites,
        [
            ("lock-table", 6, 5),
            ("nowait", 6, 52),
            ("advisory-lock", 7, 12),
            ("advisory-lock", 11, 8),
        ]
    );
    assert_eq!(
        facts.lock_inventory[2].fingerprint,
        facts.lock_inventory[3].fingerprint
    );
}

#[test]
fn advisory_facts_require_a_matching_call_at_their_reported_location() {
    let sql = fixture("advisory-alignment.sql");
    let (original, shifted) = sql.split_once("-- Alternative source layout\n").unwrap();
    let facts = extract_sql_statement_facts(original);
    assert_eq!(facts.function_calls.len(), 4);
    let original = super::PreparedSql::new(original);
    let mirrored = super::Collector::new(&original)
        .finish(facts.function_calls.iter().chain(&facts.function_calls));
    assert_eq!(mirrored, facts.lock_inventory);
    let shifted = super::PreparedSql::new(shifted);
    let mismatched = super::Collector::new(&shifted).finish(&facts.function_calls);
    assert!(mismatched.is_empty());
}

#[test]
fn reports_root_routine_declarations_as_inventory_gaps_even_when_parsing_succeeds() {
    let sql = fixture("routines.sql");
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.lock_inventory_unanalyzed, [3, 7]);
    assert_eq!(facts.lock_inventory.len(), 1);
    assert_eq!(facts.lock_inventory[0].line, 12);
    let narrow = extract_sql_conflict_policy_facts(&sql, &[]);
    assert!(narrow.lock_inventory_unanalyzed.is_empty());
}
