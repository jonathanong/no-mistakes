use super::{catalog, fixture_root};
use crate::codebase::postgres::statements::extract_sql_statement_facts_with_json_writes;
use crate::codebase::postgres::statements::SqlLocationDemand;
use crate::codebase::rules::postgres_bounded_statements::{
    column_bounds::ColumnBounds, evaluate, view_bounds::ViewBounds, Options,
};

#[test]
fn independent_bodies_do_not_borrow_outer_caps_or_predicates() {
    let sql =
        std::fs::read_to_string(fixture_root().join("../materialized-bodies/fixture/bodies.sql"))
            .unwrap();
    let extract = |enabled| {
        extract_sql_statement_facts_with_json_writes(
            &sql,
            true,
            None,
            SqlLocationDemand::None,
            false,
            false,
            enabled,
        )
    };
    let off = extract(false);
    let mut on = extract(true);
    assert!(!on.parse_failed);
    let catalog = catalog();
    let options: Options =
        serde_yaml::from_str("columnBounds: {orders.account_id: {reason: bounded account}} ")
            .unwrap();
    let mut views = ViewBounds::compile(&options.view_bounds, &catalog).unwrap();
    views.keys = ColumnBounds::compile(&options.column_bounds, &catalog).unwrap();
    let bodies = on
        .bounds
        .iter()
        .filter(|bound| bound.body.is_some())
        .collect::<Vec<_>>();
    assert_eq!(bodies.len(), 24);
    assert!(format!("{:?}", bodies[0]).contains("MATERIALIZED CTE candidates"));
    let counts = bodies
        .iter()
        .map(|body| evaluate::offenders_with_views(body, &catalog, &views).len())
        .collect::<Vec<_>>();
    assert_eq!(
        counts,
        [1, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0]
    );
    on.bounds.retain(|bound| bound.body.is_none());
    assert_eq!(on, off);
}

#[test]
fn standalone_rule_entrypoint_requests_body_facts() {
    let root = fixture_root().join("../materialized-bodies/fixture");
    let config = super::config("schemaCatalogPath: schema.json\nexecutorNames: []\ninclude: [bodies.sql]\nsqlInclude: [bodies.sql]\nmaterializedBodies: report\ncolumnBounds: {orders.account_id: {reason: bounded account}}");
    let findings =
        super::super::check_with_files(&root, &config, &[root.join("bodies.sql")]).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.line == 1
                && finding.message.contains("MATERIALIZED CTE candidates")),
        "{findings:?}"
    );
}

#[test]
fn table_bodies_keep_distinct_sites_and_quoted_relation_identity() {
    use crate::codebase::postgres::SqlBoundItemKind;
    let sql =
        std::fs::read_to_string(fixture_root().join("../materialized-bodies/fixture/table.sql"))
            .unwrap();
    let facts = extract_sql_statement_facts_with_json_writes(
        &sql,
        true,
        None,
        SqlLocationDemand::None,
        false,
        false,
        true,
    );
    let bodies = facts
        .bounds
        .iter()
        .filter(|bound| bound.body.is_some())
        .collect::<Vec<_>>();
    assert_eq!(bodies.len(), 4, "{facts:?}");
    let names = bodies
        .iter()
        .flat_map(|body| &body.query.items)
        .filter_map(|item| match &item.kind {
            SqlBoundItemKind::Table(name) => Some(name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(names, ["\"Orders\"", "accounts", "\"Orders\"", "accounts"]);
}
