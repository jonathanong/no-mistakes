use super::*;
use crate::codebase::postgres::statements::{
    extract_sql_statement_facts_with_recovered_placeholders, extract_sql_variant_statement_facts,
    SqlFactSite,
};

#[test]
fn prepared_provenance_preserves_statement_fact_bytes_across_the_sql_fixture_corpus() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres-facts");
    let manifest =
        std::fs::read_to_string(root.join("embedded/variants-provenance-corpus.txt")).unwrap();
    assert!(!manifest.is_empty());
    for name in manifest.lines().filter(|name| !name.starts_with('#')) {
        let sql = std::fs::read_to_string(root.join(name)).unwrap();
        for collect_bounds in [false, true] {
            let baseline =
                extract_sql_statement_facts_with_recovered_placeholders(&sql, collect_bounds, &[]);
            let mut projected = extract_sql_variant_statement_facts(&sql, collect_bounds, &[]);
            assert!(projected.variant_locations.take().is_some(), "{name}");
            assert_eq!(
                format!("{baseline:#?}").as_bytes(),
                format!("{projected:#?}").as_bytes(),
                "{name}: collect_bounds={collect_bounds}"
            );
        }
    }
}

#[test]
fn nested_branch_provenance_keeps_rich_sql_occurrences_on_their_physical_tokens() {
    let name = "variants-provenance.ts";
    let file = file_facts(name);
    let source = std::fs::read_to_string(&file.path).unwrap();
    assert_eq!(file.calls.len(), 1);
    assert_eq!(file.calls[0].variants.len(), 2);
    let prepared = crate::codebase::postgres::collect::dml::embedded_call_facts(&file, true, false);
    assert_eq!(prepared.len(), 2);
    let first = &prepared[0];
    assert!(!first.parse_failed);
    assert!(format!("{first:#?}").contains("variant_locations"));
    assert!(first.setting_uses.len() >= 3);
    assert!(!first.inserts.is_empty());
    assert!(!first.updates.is_empty());
    assert!(!first.deletes.is_empty());
    assert!(!first.returning_stars.is_empty());
    assert!(!first.mutation_column_uses.is_empty());
    for (arm, facts) in prepared.iter().enumerate() {
        let locations = facts.variant_locations.as_ref().unwrap();
        assert_eq!(locations.call_index, 0);
        assert_eq!(locations.variant_index, arm);
        for (site, position) in &locations.positions {
            let offset = position
                .source_offset
                .unwrap_or_else(|| panic!("{site:?}: {position:?}"));
            assert_eq!(
                position.source_line,
                source[..offset]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count()
                    + 1,
                "{site:?}"
            );
        }
    }
    let locations = first.variant_locations.as_ref().unwrap();
    let named_star = source.find("record => u.*").unwrap() + "record => ".len();
    assert!(first
        .returning_stars
        .iter()
        .enumerate()
        .any(|(index, star)| {
            star.within_function.as_deref() == Some("row_to_json")
                && locations
                    .position(SqlFactSite::ReturningStar(index))
                    .and_then(|position| position.source_offset)
                    == Some(named_star)
        }));
    for (index, setting) in first.setting_uses.iter().enumerate() {
        let offset = locations
            .position(SqlFactSite::Setting(index))
            .unwrap()
            .source_offset
            .unwrap();
        assert!(
            source[offset..]
                .trim_start_matches('\'')
                .to_ascii_lowercase()
                .starts_with(&setting.name)
                || source[offset..].starts_with("TIME ZONE"),
            "{setting:?}: {}",
            &source[offset..offset + 25]
        );
    }
    assert!(locations
        .positions
        .keys()
        .any(|site| matches!(site, SqlFactSite::WriteColumn(_, _))));
    let offset = locations
        .position(SqlFactSite::Conflict(0))
        .unwrap()
        .source_offset
        .unwrap();
    assert!(source[offset..].starts_with("ON CONFLICT"));
    let offset = locations
        .position(SqlFactSite::Lock(0))
        .unwrap()
        .source_offset
        .unwrap();
    assert!(source[offset..].starts_with("FOR UPDATE"));
}

#[test]
fn setting_provenance_accepts_static_names_and_ignores_similarly_named_expressions() {
    let file = file_facts("variants-settings.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    let prepared =
        crate::codebase::postgres::collect::dml::embedded_call_facts(&file, false, false);
    assert_eq!(prepared.len(), 2);
    // Valid ALTER DATABASE/SYSTEM settings exercise recovery beyond this parser's grammar.
    assert!(prepared[0].parse_failed);
    let facts = &prepared[0];
    assert_eq!(facts.setting_uses.len(), 6);
    let locations = facts.variant_locations.as_ref().unwrap();
    for (index, expected) in [
        "E'statement_timeout'",
        "$setting$lock_timeout$setting$",
        "'lock_timeout', 'actual'",
        "app.trace_id",
        "statement_timeout = '5s'",
        "lock_timeout = '6s'",
    ]
    .into_iter()
    .enumerate()
    {
        let offset = locations
            .position(SqlFactSite::Setting(index))
            .unwrap()
            .source_offset
            .unwrap();
        assert_eq!(offset, source.find(expected).unwrap(), "{expected}");
    }
    assert!(prepared[1].setting_uses.is_empty());
}

#[test]
fn bound_pin_subqueries_keep_nested_table_occurrences_and_qualified_lookup() {
    use crate::codebase::postgres::SqlPinSource;

    let file = file_facts("variants-bound-pins.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    let prepared = crate::codebase::postgres::collect::dml::embedded_call_facts(&file, true, false);
    assert_eq!(prepared.len(), 2);
    let facts = &prepared[0];
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 2);
    assert!(matches!(
        facts.bounds[0].query.items[0].pins[0].source,
        SqlPinSource::Query(_)
    ));
    assert!(matches!(
        facts.bounds[1].query.items[0].pins[0].source,
        SqlPinSource::ReadQuery(_)
    ));
    let locations = facts.variant_locations.as_ref().unwrap();
    for bound in 0..2 {
        for (needle, table) in [("public.accounts", "accounts"), ("orders o", "orders")] {
            let offset = source.match_indices(needle).nth(bound).unwrap().0;
            let line = source[..offset]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count()
                + 1;
            let sites = locations
                .bound_table_sites(bound, table, line)
                .collect::<Vec<_>>();
            assert_eq!(sites.len(), 1, "bound={bound}, table={table}");
            assert_eq!(sites[0].1.source_offset, Some(offset));
            // Public provenance may be trimmed independently of prepared facts.
            // A lookup must skip an occurrence whose position was removed.
            let mut trimmed = locations.clone();
            trimmed.positions.remove(&sites[0].0);
            assert_eq!(trimmed.bound_table_sites(bound, table, line).count(), 0);
            assert_eq!(
                locations.bound_table_sites(bound, "missing", line).count(),
                0
            );
            assert_eq!(
                locations.bound_table_sites(bound, table, line + 10).count(),
                0
            );
            if table == "accounts" {
                assert_eq!(locations.bound_table_sites(bound, needle, line).count(), 1);
            }
        }
    }
}
