use super::query_is_correlated;
use crate::codebase::postgres::parse_postgres_sql;
use crate::codebase::postgres::statements::extract_sql_statement_facts;
use sqlparser::ast::Statement;

fn exists_ops(sql: &str) -> Vec<(bool, bool)> {
    extract_sql_statement_facts(sql)
        .selects
        .into_iter()
        .flat_map(|select| select.exists_set_operations)
        .map(|exists| (exists.restricted, exists.correlated))
        .collect()
}

#[test]
fn uncorrelated_union_is_not_correlated() {
    let flags = exists_ops(
        "SELECT 1 WHERE EXISTS (
            SELECT 1 FROM topics
            UNION ALL
            SELECT 1 FROM topics
         )",
    );
    assert_eq!(flags, vec![(false, false)], "{flags:?}");
}

#[test]
fn outer_qualifier_is_correlated() {
    let flags = exists_ops(
        "SELECT * FROM posts WHERE EXISTS (
            SELECT 1 FROM topics WHERE topics.post_id = posts.id
            UNION ALL
            SELECT 1 FROM tags WHERE tags.post_id = posts.id
         )",
    );
    assert_eq!(flags, vec![(false, true)], "{flags:?}");
}

#[test]
fn inner_restriction_does_not_clear_correlation() {
    let flags = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            SELECT 1 FROM topics WHERE topics.post_id = posts.id AND topics.id = $1
            UNION ALL
            SELECT 1 FROM tags WHERE tags.post_id = posts.id AND tags.id = $1
         )",
    );
    assert_eq!(flags, vec![(true, true)], "{flags:?}");
}

#[test]
fn select_list_probe_is_uncorrelated() {
    let flags = exists_ops(
        "SELECT EXISTS (
            SELECT 1 FROM topics
            UNION ALL
            SELECT 1 FROM topics
         ) AS has_any",
    );
    assert_eq!(flags, vec![(false, false)], "{flags:?}");
}

#[test]
fn select_list_and_having_outer_refs_are_correlated() {
    let projection = exists_ops(
        "SELECT EXISTS (
            SELECT 1 FROM topics WHERE topics.post_id = posts.id
            UNION ALL
            SELECT 1 FROM tags WHERE tags.post_id = posts.id
         ) FROM posts",
    );
    assert_eq!(projection, vec![(false, true)], "{projection:?}");
    let having = exists_ops(
        "SELECT 1 FROM posts GROUP BY posts.id HAVING EXISTS (
            SELECT * FROM topics WHERE topics.post_id = posts.id
            UNION ALL
            SELECT * FROM tags WHERE tags.post_id = posts.id
         )",
    );
    assert_eq!(having, vec![(false, true)], "{having:?}");
}

#[test]
fn alias_cte_join_and_wrapper_shapes() {
    let aliased = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            SELECT 1 FROM topics t WHERE t.post_id = posts.id
            UNION ALL
            SELECT 1 FROM tags WHERE tags.post_id = posts.id
         )",
    );
    assert_eq!(aliased, vec![(false, true)]);
    let cte = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            WITH x AS (SELECT 1 FROM topics WHERE topics.post_id = posts.id)
            SELECT 1 FROM x
            UNION ALL
            SELECT 1 FROM x
         )",
    );
    assert_eq!(cte, vec![(false, true)]);
    let wrapped = exists_ops(
        "SELECT 1 WHERE EXISTS (
            (SELECT 1 FROM topics UNION ALL SELECT 1 FROM topics)
         )",
    );
    assert_eq!(wrapped, vec![(false, false)]);
    let derived = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            SELECT 1 FROM (
                SELECT 1 FROM topics UNION ALL SELECT 1 FROM tags
            ) candidate
         )",
    );
    assert!(derived.is_empty(), "{derived:?}");
}

#[test]
fn join_cast_in_subquery_and_values_arms() {
    let join = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            SELECT 1 FROM topics JOIN tags ON tags.post_id = posts.id
            UNION ALL
            SELECT 1 FROM topics LEFT OUTER JOIN tags ON tags.post_id = posts.id
         )",
    );
    assert_eq!(join, vec![(false, true)]);
    let outer = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            SELECT 1 FROM topics LEFT JOIN tags ON tags.post_id = posts.id
            UNION ALL
            SELECT 1 FROM topics RIGHT OUTER JOIN tags ON tags.post_id = posts.id
         )",
    );
    assert_eq!(outer, vec![(false, true)]);
    let casted = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            SELECT 1 FROM topics WHERE topics.post_id = CAST(posts.id AS int)
            UNION ALL
            SELECT 1 FROM tags WHERE NOT (tags.post_id = posts.id)
         )",
    );
    assert_eq!(casted, vec![(false, true)]);
    let nested_in = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            SELECT 1 FROM topics WHERE topics.id IN (SELECT posts.id)
            UNION ALL
            SELECT 1 FROM tags
         )",
    );
    assert_eq!(nested_in, vec![(false, true)]);
    let values = exists_ops("SELECT 1 WHERE EXISTS (VALUES (1) UNION ALL VALUES (2))");
    assert_eq!(values, vec![(false, false)], "{values:?}");
    let on_exists = exists_ops(
        "SELECT 1 FROM posts JOIN extra ON EXISTS (
            SELECT 1 FROM topics WHERE topics.post_id = posts.id
            UNION ALL
            SELECT 1 FROM tags WHERE tags.post_id = posts.id
         )",
    );
    assert_eq!(on_exists, vec![(false, true)]);
}

#[test]
fn nested_join_cross_join_and_full_join_are_walked() {
    let nested = exists_ops(
        "SELECT 1 FROM posts CROSS JOIN extra WHERE EXISTS (
            SELECT 1 FROM (topics CROSS JOIN tags)
            WHERE topics.post_id = posts.id
            UNION ALL
            SELECT 1 FROM generate_series(1, 2) AS g
         )",
    );
    assert_eq!(nested, vec![(false, true)], "{nested:?}");
    let full = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            SELECT 1 FROM topics FULL JOIN tags ON tags.post_id = posts.id
            UNION ALL
            SELECT 1 FROM topics RIGHT JOIN tags ON tags.post_id = posts.id
         )",
    );
    assert_eq!(full, vec![(false, true)]);
}

#[test]
fn query_is_correlated_reads_direct_select() {
    let Statement::Query(query) = parse_postgres_sql(
        "SELECT 1 FROM topics WHERE topics.post_id = posts.id
         UNION ALL
         SELECT 1 FROM tags WHERE tags.post_id = posts.id",
    )
    .unwrap()
    .pop()
    .unwrap() else {
        panic!("query");
    };
    assert!(query_is_correlated(&query));
}

#[test]
fn coalesce_case_between_and_schema_qualified_refs() {
    let wrapped = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            SELECT 1 FROM topics WHERE topics.post_id = COALESCE(posts.id, 0)
            UNION ALL
            SELECT 1 FROM tags WHERE tags.post_id BETWEEN posts.id AND posts.id
         )",
    );
    assert_eq!(wrapped, vec![(false, true)], "{wrapped:?}");
    let distinct = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            SELECT 1 FROM topics WHERE topics.post_id IS DISTINCT FROM posts.id
            UNION ALL
            SELECT 1 FROM tags WHERE tags.post_id IS NOT NULL
         )",
    );
    assert_eq!(distinct, vec![(false, true)], "{distinct:?}");
    let schema_local = exists_ops(
        "SELECT 1 WHERE EXISTS (
            SELECT 1 FROM public.topics WHERE public.topics.id = 1
            UNION ALL
            SELECT 1 FROM public.tags WHERE public.tags.id = 2
         )",
    );
    assert_eq!(schema_local, vec![(true, false)], "{schema_local:?}");
}

#[test]
fn aliased_inner_table_hides_base_name() {
    let flags = exists_ops(
        "SELECT 1 FROM topics WHERE EXISTS (
            SELECT 1 FROM topics AS inner_topics
            WHERE inner_topics.parent_id = topics.id
            UNION ALL
            SELECT 1 FROM tags WHERE tags.topic_id = topics.id
         )",
    );
    assert_eq!(flags, vec![(false, true)], "{flags:?}");
}

#[test]
fn select_list_case_wrapped_exists_is_collected() {
    let flags = exists_ops(
        "SELECT CASE WHEN EXISTS (
            SELECT 1 FROM topics WHERE topics.post_id = posts.id
            UNION ALL
            SELECT 1 FROM tags WHERE tags.post_id = posts.id
         ) THEN 1 END FROM posts",
    );
    assert_eq!(flags, vec![(false, true)], "{flags:?}");
}

#[test]
fn function_inlist_star_and_natural_join_shapes() {
    let coalesced = exists_ops(
        "SELECT COALESCE(EXISTS (
            SELECT 1 FROM topics WHERE topics.post_id = posts.id
            UNION ALL
            SELECT 1 FROM tags WHERE tags.post_id = posts.id
         ), false) FROM posts",
    );
    assert_eq!(coalesced, vec![(false, true)], "{coalesced:?}");
    let listed = exists_ops(
        "SELECT * FROM posts WHERE EXISTS (
            SELECT 1 FROM topics WHERE topics.post_id IN (posts.id, 0)
            UNION ALL
            SELECT 1 FROM tags WHERE tags.post_id IS TRUE
         )",
    );
    assert_eq!(listed, vec![(false, true)], "{listed:?}");
    let natural = exists_ops(
        "SELECT 1 WHERE EXISTS (
            SELECT 1 FROM topics NATURAL JOIN tags
            UNION ALL
            SELECT 1 FROM topics
         )",
    );
    assert_eq!(natural, vec![(false, false)], "{natural:?}");
}

#[test]
fn like_in_subquery_table_args_group_by_and_quoted_idents() {
    let liked = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            SELECT 1 FROM topics WHERE topics.name LIKE posts.pattern
            UNION ALL
            SELECT 1 FROM tags WHERE tags.name LIKE posts.pattern
         )",
    );
    assert_eq!(liked, vec![(false, true)], "{liked:?}");
    let in_left = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            SELECT 1 FROM topics WHERE posts.id IN (SELECT post_id FROM allowed)
            UNION ALL
            SELECT 1 FROM tags WHERE posts.id IN (SELECT post_id FROM allowed)
         )",
    );
    assert_eq!(in_left, vec![(false, true)], "{in_left:?}");
    let table_args = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            SELECT 1 FROM jsonb_array_elements(posts.tags) AS tag
            UNION ALL
            SELECT 1 FROM jsonb_array_elements(posts.tags) AS tag
         )",
    );
    assert_eq!(table_args, vec![(false, true)], "{table_args:?}");
    let grouped = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            SELECT count(*) FROM topics GROUP BY posts.id
            UNION ALL
            SELECT count(*) FROM tags GROUP BY posts.id
         )",
    );
    assert_eq!(grouped, vec![(false, true)], "{grouped:?}");
    let quoted = exists_ops(
        r#"SELECT 1 FROM "Posts" WHERE EXISTS (
            SELECT 1 FROM topics AS posts WHERE topics.post_id = "Posts".id
            UNION ALL
            SELECT 1 FROM tags WHERE tags.post_id = "Posts".id
         )"#,
    );
    assert_eq!(quoted, vec![(false, true)], "{quoted:?}");
    let values = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            VALUES (posts.id)
            UNION ALL
            VALUES (1)
         )",
    );
    assert_eq!(values, vec![(false, true)], "{values:?}");
    let grouped_exists = exists_ops(
        "SELECT 1 FROM posts GROUP BY EXISTS (
            SELECT 1 FROM topics WHERE topics.post_id = posts.id
            UNION ALL
            SELECT 1 FROM tags WHERE tags.post_id = posts.id
         )",
    );
    assert_eq!(grouped_exists, vec![(false, true)], "{grouped_exists:?}");
    let wildcard = exists_ops(
        "SELECT 1 FROM posts WHERE EXISTS (
            SELECT posts.* FROM topics
            UNION ALL
            SELECT posts.* FROM tags
         )",
    );
    assert_eq!(wildcard, vec![(false, true)], "{wildcard:?}");
}

#[test]
fn query_is_correlated_walks_remaining_set_and_join_shapes() {
    let Statement::Query(query) = parse_postgres_sql(
        "WITH extra AS (SELECT 1)
         SELECT topics.* FROM topics
         INNER JOIN LATERAL (SELECT posts.id) nested ON nested.id = topics.id
         WHERE EXISTS (SELECT 1 FROM extra)
         GROUP BY ALL",
    )
    .unwrap()
    .pop()
    .unwrap() else {
        panic!("query");
    };
    assert!(query_is_correlated(&query));
    let Statement::Query(values) = parse_postgres_sql("VALUES (posts.id), (1)")
        .unwrap()
        .pop()
        .unwrap()
    else {
        panic!("values");
    };
    assert!(query_is_correlated(&values));
    let Statement::Query(join) = parse_postgres_sql(
        "SELECT 1 FROM topics
         JOIN tags ON tags.post_id = outer_posts.id
         LEFT OUTER JOIN extra ON extra.post_id = outer_posts.id
         RIGHT OUTER JOIN other ON extra.id = other.id
         FULL OUTER JOIN more ON more.id = outer_posts.id",
    )
    .unwrap()
    .pop()
    .unwrap() else {
        panic!("join");
    };
    assert!(query_is_correlated(&join));
    for sql in [
        "SELECT 1 FROM (SELECT posts.id) AS derived",
        "SELECT 1 FROM generate_series(1, posts.n) AS g",
        "SELECT 1 WHERE posts.id IN (SELECT id FROM topics)",
        "SELECT (SELECT posts.id)",
    ] {
        let Statement::Query(query) = parse_postgres_sql(sql).unwrap().pop().unwrap() else {
            panic!("{sql}");
        };
        assert!(query_is_correlated(&query), "{sql} should be correlated");
    }
    let Statement::Query(wrap) =
        parse_postgres_sql("SELECT 1 FROM (SELECT 1 FROM nested JOIN extra ON true) AS wrap")
            .unwrap()
            .pop()
            .unwrap()
    else {
        panic!("wrap");
    };
    assert!(!query_is_correlated(&wrap));
}

#[test]
fn query_is_correlated_covers_parser_rare_variants() {
    use sqlparser::ast::{
        Expr, GroupByExpr, Ident, JoinOperator, SelectItem, SelectItemQualifiedWildcardKind,
        SetExpr, TableFactor, WildcardAdditionalOptions,
    };

    let Statement::Query(mut query) =
        parse_postgres_sql("SELECT topics.* FROM topics JOIN tags ON tags.id = topics.id")
            .unwrap()
            .pop()
            .unwrap()
    else {
        panic!("query");
    };
    let SetExpr::Select(select) = query.body.as_mut() else {
        panic!("select");
    };
    select.group_by = GroupByExpr::All(vec![]);
    select.projection.push(SelectItem::QualifiedWildcard(
        SelectItemQualifiedWildcardKind::Expr(Expr::Identifier(Ident::new("topics"))),
        WildcardAdditionalOptions::default(),
    ));
    let inner = select.from[0].clone();
    select.from[0].relation = TableFactor::NestedJoin {
        table_with_joins: Box::new(inner.clone()),
        alias: None,
    };
    select.from.push(inner.clone());
    select.from.last_mut().unwrap().relation = TableFactor::TableFunction {
        expr: Expr::Identifier(Ident::new("topics")),
        alias: None,
    };
    if let Some(join) = select.from[0].joins.first_mut() {
        join.join_operator = JoinOperator::Inner(sqlparser::ast::JoinConstraint::On(
            Expr::Identifier(Ident::new("topics")),
        ));
    }
    assert!(!query_is_correlated(&query));

    *query.body = SetExpr::Insert(parse_postgres_sql("COMMIT").unwrap().pop().unwrap());
    assert!(!query_is_correlated(&query));
}
