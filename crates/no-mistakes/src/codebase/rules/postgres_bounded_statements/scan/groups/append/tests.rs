use super::{only_appended, span};
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::statements::SqlVariantLocations;
use crate::codebase::postgres::SqlStatementFileFacts;

#[test]
fn missing_relation_locations_prevent_the_all_appended_proof() {
    let mut locations = SqlVariantLocations {
        original_call_line: 20,
        ..Default::default()
    };
    let mapped = SqlFactSite::BoundRelation(0, 1);
    locations.insert(SqlFactSite::Origin, 1, 1);
    locations
        .positions
        .get_mut(&SqlFactSite::Origin)
        .unwrap()
        .source_line = 10;
    locations.insert(mapped.clone(), 1, 1);
    locations.positions.get_mut(&mapped).unwrap().source_line = 5;
    locations.bound_tables.push((0, "invoices".into(), mapped));
    let mut file = SqlStatementFileFacts {
        path: "query.ts".into(),
        variant_locations: Some(locations),
        ..Default::default()
    };
    assert_eq!(only_appended(&[span(&file, 0).unwrap()]), None);
    file.variant_locations
        .as_mut()
        .unwrap()
        .append_sites
        .push(4);
    assert_eq!(only_appended(&[span(&file, 0).unwrap()]), None);
    file.variant_locations
        .as_mut()
        .unwrap()
        .first_append_position = Some((1, 1));
    assert_eq!(only_appended(&[span(&file, 0).unwrap()]), Some(20));
    file.variant_locations.as_mut().unwrap().bound_tables.push((
        0,
        "accounts".into(),
        SqlFactSite::BoundRelation(0, 0),
    ));
    let incomplete = span(&file, 0).unwrap();
    assert_eq!(incomplete.relations, [(5, 4)]);
    assert_eq!(only_appended(&[incomplete]), None);
}

fn statements(relative: &str) -> Vec<SqlStatementFileFacts> {
    use crate::codebase::postgres::embedded::{
        extract_embedded_sql_from_source, EmbeddedSqlOptions, TrustedSqlTag,
    };
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements").join(relative);
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
        TrustedSqlTag {
            module: "@example/db".into(),
            name: "sql".into(),
        },
    ]);
    let file = extract_embedded_sql_from_source(&path, &source, &options);
    crate::codebase::postgres::collect::dml::embedded_call_facts_with_locations(
        &file, true, false, true,
    )
}

#[test]
fn full_from_callers_keep_explicit_append_metadata() {
    let files = statements("../variants/shared-fragment/src/full-from.ts");
    let mut calls = Vec::new();
    for file in &files {
        let locations = file.variant_locations.as_ref().unwrap();
        if [38, 42].contains(&locations.original_call_line) {
            assert!(!locations.append_sites.is_empty());
            calls.push(only_appended(&[span(file, 0).unwrap()]).unwrap());
        }
    }
    calls.sort_unstable();
    assert_eq!(calls, [38, 42]);
}

#[test]
fn direct_escaped_newlines_and_concatenation_do_not_prove_builder_appends() {
    let files = statements("src/issue-2961-lines.mts");
    assert_eq!(files.len(), 4);
    for file in &files {
        assert!(file
            .variant_locations
            .as_ref()
            .unwrap()
            .append_sites
            .is_empty());
        assert_eq!(only_appended(&[span(file, 0).unwrap()]), None);
    }
}

#[test]
fn an_explicit_append_does_not_relocate_an_escaped_opening_relation() {
    let files = statements("../variants/shared-fragment/src/full-from.ts");
    let file = files
        .iter()
        .find(|file| file.variant_locations.as_ref().unwrap().original_call_line == 64)
        .unwrap();
    assert!(!file
        .variant_locations
        .as_ref()
        .unwrap()
        .append_sites
        .is_empty());
    assert_eq!(only_appended(&[span(file, 0).unwrap()]), None);
}

#[test]
fn append_boundaries_ignore_helper_order_and_empty_openings() {
    let files = statements("../variants/shared-fragment/src/full-from.ts");
    for line in [68, 72, 80, 84] {
        let file = files
            .iter()
            .find(|file| file.variant_locations.as_ref().unwrap().original_call_line == line)
            .unwrap();
        assert_eq!(
            only_appended(&[span(file, 0).unwrap()]),
            Some(line as usize)
        );
    }
    let physical = files
        .iter()
        .find(|file| file.variant_locations.as_ref().unwrap().original_call_line == 94)
        .unwrap();
    assert!(physical
        .variant_locations
        .as_ref()
        .unwrap()
        .first_append_position
        .is_some());
    assert_eq!(only_appended(&[span(physical, 0).unwrap()]), None);
}
