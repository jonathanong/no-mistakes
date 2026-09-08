use super::*;
use crate::codebase::postgres::{CanonicalIndex, SqlInsertSourceShape};

#[test]
fn malformed_conflict_sql_only_reports_when_the_policy_requires_analysis() {
    let catalog = SchemaCatalog::default();
    let malformed = "INSERT INTO items (id) VALUES ( ON CONFLICT (id) DO NOTHING";

    let findings = findings_for_sql("query.ts", 7, malformed, &catalog, true);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable-sql"));

    assert!(findings_for_sql("query.ts", 7, malformed, &catalog, false).is_empty());
    assert!(findings_for_sql("query.ts", 7, "not valid SQL", &catalog, true).is_empty());
}

#[test]
fn targetless_multi_row_conflicts_do_not_need_a_catalog_to_be_rejected() {
    let findings = findings_for_sql(
        "query.ts",
        7,
        "INSERT INTO items (id) SELECT id FROM pending ON CONFLICT DO NOTHING",
        &SchemaCatalog::default(),
        true,
    );
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("targetless-arbiter"));
}

#[test]
fn helper_branches_preserve_order_metadata_and_alias_fallbacks() {
    let expected = vec![CanonicalOrderKey {
        expression: "tenant_id".to_string(),
        ascending: false,
        nulls_first: true,
    }];
    assert_eq!(display_keys(&expected), "tenant_id DESC NULLS FIRST");

    let aliases =
        std::collections::BTreeMap::from([("tenant".to_string(), "tenant_id".to_string())]);
    let resolved = resolve_order_aliases(
        &[
            CanonicalOrderKey {
                expression: "tenant".to_string(),
                ascending: false,
                nulls_first: true,
            },
            CanonicalOrderKey {
                expression: "id".to_string(),
                ascending: true,
                nulls_first: false,
            },
        ],
        &aliases,
    );
    assert_eq!(resolved[0].expression, "tenant_id");
    assert_eq!(resolved[1].expression, "id");

    let index = CanonicalIndex {
        name: "items_key".to_string(),
        constraint_backed: false,
        keys: expected.clone(),
        predicate: None,
    };
    assert!(target_matches_catalog(&["tenant_id".to_string()], &index));
    assert!(!target_matches_catalog(&["id".to_string()], &index));
    assert!(!target_matches_catalog(
        &["tenant_id".to_string(), "id".to_string()],
        &index
    ));

    assert!(expected_source_order(
        &index,
        &SqlInsertSourceShape {
            multi_row: true,
            order: None,
            projections: None,
            order_aliases: std::collections::BTreeMap::new(),
        }
    )
    .is_none());
    assert!(expected_source_order(
        &index,
        &SqlInsertSourceShape {
            multi_row: true,
            order: None,
            projections: Some(std::collections::BTreeMap::from([(
                "id".to_string(),
                "source_id".to_string(),
            )])),
            order_aliases: std::collections::BTreeMap::new(),
        }
    )
    .is_none());
    assert!(contains_insert_conflict(
        "INSERT INTO items ON CONFLICT DO NOTHING"
    ));
    assert!(!contains_insert_conflict("INSERT INTO items"));
    assert!(contains_insert("insert into items"));
    assert!(!contains_insert("reinsertion"));
}
