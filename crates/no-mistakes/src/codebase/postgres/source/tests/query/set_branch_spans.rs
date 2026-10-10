use super::*;

#[test]
fn set_branches_keep_exact_source_without_confusing_aliases_or_suffixes() {
    let source = fixture("query-set-branch-spans.sql");
    let queries = queries("query-set-branch-spans.sql");
    assert_eq!(queries.len(), 13);
    let slices = |query: &PostgresSqlQuery| {
        assert!(query.complete, "{:?}", query.unsupported);
        assert!(query.unsupported.is_empty());
        query
            .scopes
            .iter()
            .filter(|scope| scope.clause == PostgresSqlQueryClause::SetBranch)
            .map(|scope| {
                let span = scope.span.as_ref().expect("complete branch has a span");
                source[span.start.offset..span.end.offset].to_owned()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(slices(&queries[0]), ["SELECT 'é'", "SELECT now()"]);
    assert_eq!(
        slices(&queries[1]),
        ["(SELECT now())", "SELECT now()", "SELECT now()"]
    );
    assert_eq!(
        slices(&queries[2]),
        ["SELECT now()", "(SELECT now())", "SELECT now()"]
    );
    assert_eq!(slices(&queries[3]), ["SELECT 1", "SELECT 2 AS order"]);
    assert_eq!(
        slices(&queries[4]),
        ["SELECT 1", "SELECT t.limit FROM metrics AS t"]
    );
    assert_eq!(slices(&queries[5]), ["SELECT now()", "SELECT now()"]);
    assert_eq!(
        slices(&queries[6]),
        [
            "SELECT now() UNION ALL SELECT now()",
            "SELECT now()",
            "SELECT now()",
            "SELECT now()"
        ]
    );
    let cte = slices(&queries[7]);
    assert_eq!(cte, ["SELECT now()", "SELECT now()"]);
    assert_eq!(slices(&queries[8]), ["SELECT now()", "SELECT now()"]);
    assert_eq!(slices(&queries[9]), ["SELECT now()", "SELECT now()"]);
    assert_eq!(
        slices(&queries[10]),
        ["SELECT 1", "SELECT t.fetch FROM metrics AS t"]
    );
    let double_left = slices(&queries[11]);
    assert!(
        double_left.contains(&"((SELECT now()))".to_owned()),
        "{double_left:?}"
    );
    let double_right = slices(&queries[12]);
    assert!(
        double_right.contains(&"((SELECT now()))".to_owned()),
        "{double_right:?}"
    );
}

#[test]
fn long_left_associated_set_chain_keeps_every_leaf_span() {
    let source = fixture("query-set-branch-chain.sql");
    let queries = queries("query-set-branch-chain.sql");
    assert_eq!(queries.len(), 1);
    let query = &queries[0];
    assert!(query.complete, "{:?}", query.unsupported);
    let branches = query
        .scopes
        .iter()
        .filter(|scope| scope.clause == PostgresSqlQueryClause::SetBranch)
        .map(|scope| {
            let span = scope.span.as_ref().expect("branch span");
            &source[span.start.offset..span.end.offset]
        })
        .collect::<Vec<_>>();
    assert_eq!(branches.len(), 46); // 24 leaves and 22 intermediate set nodes.
    assert_eq!(
        branches
            .iter()
            .filter(|slice| **slice == "SELECT now()")
            .count(),
        24
    );
    assert!(branches
        .iter()
        .all(|slice| slice.starts_with("SELECT now()") && slice.ends_with("SELECT now()")));
}
