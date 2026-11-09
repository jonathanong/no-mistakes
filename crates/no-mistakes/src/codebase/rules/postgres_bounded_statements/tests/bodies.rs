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
    assert_eq!(bodies.len(), 16);
    assert!(format!("{:?}", bodies[0]).contains("MATERIALIZED CTE candidates"));
    let counts = bodies
        .iter()
        .map(|body| evaluate::offenders_with_views(body, &catalog, &views).len())
        .collect::<Vec<_>>();
    assert_eq!(counts, [1, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 1, 0]);
    let scalar_counts: Vec<_> = on
        .bounds
        .iter()
        .filter(|fact| {
            fact.body.is_none() && [53, 54, 58, 59].contains(&fact.line) && fact.column > 1
        })
        .map(|fact| {
            (
                fact.line,
                evaluate::offenders_with_views(fact, &catalog, &views).len(),
            )
        })
        .collect();
    assert_eq!(scalar_counts, [(53, 1), (54, 0), (58, 1), (59, 0)]);
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
    assert_eq!(bodies.len(), 2, "{facts:?}");
    let names = bodies
        .iter()
        .flat_map(|body| &body.query.items)
        .filter_map(|item| match &item.kind {
            SqlBoundItemKind::Table(name) => Some(name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(names, ["accounts", "accounts"]);
    let scalars: Vec<_> = facts
        .bounds
        .iter()
        .filter(|fact| fact.body.is_none() && fact.column > 1)
        .collect();
    assert_eq!(scalars.len(), 2);
    for (scalar, capped) in scalars.iter().zip([false, true]) {
        assert!(
            matches!(&scalar.query.items[0].kind, SqlBoundItemKind::Table(name) if name == "\"Orders\"")
        );
        assert_eq!(scalar.query.capped, capped);
    }
}

#[test]
fn deeply_nested_derived_bodies_are_projected_once() {
    let sql = std::fs::read_to_string(fixture_root().join("../materialized-bodies/deep/query.sql"))
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
    assert!(!facts.parse_failed);
    assert_eq!(
        facts
            .bounds
            .iter()
            .filter(|bound| bound.body.is_some())
            .count(),
        40
    );
    let catalog = catalog();
    let views = ViewBounds::compile(&Default::default(), &catalog).unwrap();
    let counts = facts
        .bounds
        .iter()
        .filter(|bound| bound.body.is_some())
        .map(|bound| evaluate::offenders_with_views(bound, &catalog, &views).len())
        .collect::<Vec<_>>();
    assert_eq!(&counts[..20], &[1; 20]);
    assert_eq!(&counts[20..], &[0; 20]);
}

#[test]
fn body_statement_filters_follow_the_owning_execution() {
    use crate::codebase::postgres::SqlBoundKind;
    let sql =
        std::fs::read_to_string(fixture_root().join("../materialized-bodies/owners/query.sql"))
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
    assert!(!facts.parse_failed);
    let catalog = catalog();
    let views = ViewBounds::compile(&Default::default(), &catalog).unwrap();
    for (kind, count, scalar_count) in [
        (SqlBoundKind::Update, 2, 1),
        (SqlBoundKind::Delete, 3, 0),
        (SqlBoundKind::Select, 1, 0),
    ] {
        assert_eq!(
            facts
                .bounds
                .iter()
                .filter(|fact| fact.body.is_some() && fact.kind == kind)
                .flat_map(|fact| evaluate::offenders_with_views(fact, &catalog, &views))
                .count(),
            count
        );
        assert_eq!(
            facts
                .bounds
                .iter()
                .filter(|fact| fact.body.is_none()
                    && fact.target.is_none()
                    && fact.column > 1
                    && fact.kind == kind)
                .flat_map(|fact| evaluate::offenders_with_views(fact, &catalog, &views))
                .count(),
            scalar_count
        );
    }
}
