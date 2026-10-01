use super::super::{
    catalog_finding, require_catalog_path, AllowEntry, AllowList, CatalogObjectRef,
};
use crate::codebase::rules::RuleFinding;
use std::str::FromStr;

#[test]
fn object_refs_round_trip_and_reject_bad_syntax() {
    let refs = [
        "table:accounts",
        "column:accounts.id",
        "column:public.orders.id",
        "index:accounts.accounts_pkey",
        "index:public.orders.orders_pkey",
        "trigger:accounts.trigger_accounts_touch",
        "trigger:public.orders.touch",
        "constraint:accounts.accounts_owner_fk",
        "constraint:public.orders.orders_fk",
        "function:fn_touch(col text)",
        "enum:account_plans",
        "view:active_accounts",
        "materialized-view:account_counts",
    ];
    for raw in refs {
        let parsed = CatalogObjectRef::from_str(raw).unwrap();
        assert_eq!(parsed.to_string(), raw);
    }
    for raw in [
        "",
        "accounts",
        "nope:accounts",
        "table:",
        "table:   ",
        "column:accounts",
        "column:.id",
        "column:accounts.",
        "column:public.orders.",
        "index:public.",
        "function:",
        "materialized-view:",
    ] {
        assert!(CatalogObjectRef::from_str(raw).is_err(), "{raw}");
    }
}

#[test]
fn catalog_findings_use_the_object_ref_as_the_locator() {
    let finding = catalog_finding(
        "schema-catalog-test-rule",
        r"db\schema.json",
        &CatalogObjectRef::Column {
            table: "orders".to_string(),
            column: "status".to_string(),
        },
        "needs a trigger",
    );
    assert_eq!(
        finding,
        RuleFinding {
            rule: "schema-catalog-test-rule".to_string(),
            file: "db/schema.json".to_string(),
            line: 1,
            message: "db/schema.json: column:orders.status: needs a trigger".to_string(),
            import: None,
            target: Some("column:orders.status".to_string()),
        }
    );
}

#[test]
fn require_catalog_path_rejects_an_empty_path() {
    let error = require_catalog_path("schema-catalog-test-rule", "").unwrap_err();
    assert_eq!(
        error.to_string(),
        "schema-catalog-test-rule option schemaCatalogPath: required"
    );
    assert!(require_catalog_path("schema-catalog-test-rule", "db/schema.json").is_ok());
}

#[test]
fn allow_list_suppresses_matches_and_reports_stale_entries() {
    let list = AllowList::compile(
        "schema-catalog-test-rule",
        vec![
            AllowEntry {
                object: "table:accounts".to_string(),
                reason: "kept".to_string(),
            },
            AllowEntry {
                object: "column:accounts.id".to_string(),
                reason: "not produced".to_string(),
            },
        ],
    )
    .unwrap();
    let findings = list.apply(
        "db/schema.json",
        vec![
            finding("table:accounts"),
            finding("table:accounts"),
            finding("table:events"),
            RuleFinding {
                target: None,
                ..finding("table:events")
            },
        ],
    );
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.message.as_str())
            .collect::<Vec<_>>(),
        [
            "db/schema.json: table:events: checked",
            "db/schema.json: table:events: checked",
            "db/schema.json: stale schema-catalog-test-rule allow entry: column:accounts.id",
        ]
    );
    assert_eq!(findings[2].target.as_deref(), Some("column:accounts.id"));
    assert_eq!(findings[2].line, 1);
    assert_eq!(findings[2].file, "db/schema.json");
}

#[test]
fn allow_list_rejects_empty_invalid_and_duplicate_entries() {
    let empty = AllowList::compile(
        "schema-catalog-test-rule",
        vec![AllowEntry {
            object: "table:accounts".to_string(),
            reason: "   ".to_string(),
        }],
    )
    .unwrap_err();
    assert_eq!(
        empty.to_string(),
        "schema-catalog-test-rule option allow: entry table:accounts needs a reason"
    );
    let invalid = AllowList::compile(
        "schema-catalog-test-rule",
        vec![AllowEntry {
            object: "column:".to_string(),
            reason: "because".to_string(),
        }],
    )
    .unwrap_err();
    assert_eq!(
        invalid.to_string(),
        "schema-catalog-test-rule option allow: invalid object ref column:"
    );
    let duplicate = AllowList::compile(
        "schema-catalog-test-rule",
        vec![
            AllowEntry {
                object: "table:accounts".to_string(),
                reason: "one".to_string(),
            },
            AllowEntry {
                object: "table:accounts".to_string(),
                reason: "two".to_string(),
            },
        ],
    )
    .unwrap_err();
    assert_eq!(
        duplicate.to_string(),
        "schema-catalog-test-rule option allow: duplicate entry table:accounts"
    );
}

fn finding(target: &str) -> RuleFinding {
    RuleFinding {
        rule: "schema-catalog-test-rule".to_string(),
        file: "db/schema.json".to_string(),
        line: 1,
        message: format!("db/schema.json: {target}: checked"),
        import: None,
        target: Some(target.to_string()),
    }
}
