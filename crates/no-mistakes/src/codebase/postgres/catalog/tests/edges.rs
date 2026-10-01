use super::super::{
    GeneratedKind, PartitionKeyElement, PartitionStrategy, TriggerEvent, TriggerTiming,
};
use super::load_fixture;

#[test]
fn edge_fixtures_cover_trigger_partition_and_function_shapes() {
    let catalog = load_fixture("edges.json").unwrap();
    let order = catalog.table("Order").unwrap();
    assert_eq!(
        order
            .columns
            .iter()
            .map(|column| column.name.as_str())
            .collect::<Vec<_>>(),
        ["cached_name", "legacy_id", "tags"]
    );
    assert_eq!(order.columns[0].generated, Some(GeneratedKind::Stored));
    assert_eq!(
        order.columns[0].generated_expression.as_deref(),
        Some("lower(display_name)")
    );
    assert_eq!(order.columns[1].identity.as_deref(), Some("always"));
    assert_eq!(order.columns[2].data_type, "text[]");

    let instead = order
        .triggers
        .iter()
        .find(|trigger| trigger.name == "quoted_instead")
        .unwrap();
    assert_eq!(instead.timing, TriggerTiming::InsteadOf);
    assert_eq!(instead.events, vec![TriggerEvent::Delete]);
    assert!(!instead.for_each_row);
    assert_eq!(instead.function, "fn_X");
    assert_eq!(
        instead.arguments,
        vec!["it's".to_string(), "a,b".to_string()]
    );
    assert_eq!(instead.when, None);

    let truncate = order
        .triggers
        .iter()
        .find(|trigger| trigger.name == "truncate_row")
        .unwrap();
    assert_eq!(truncate.timing, TriggerTiming::After);
    assert_eq!(truncate.events, vec![TriggerEvent::Truncate]);
    assert!(truncate.for_each_row);
    assert_eq!(truncate.when.as_deref(), Some("(\"a)\") = ')'"));
    assert_eq!(truncate.function, "fn_trunc");
    assert!(truncate.arguments.is_empty());

    let multi = order
        .triggers
        .iter()
        .find(|trigger| trigger.name == "multi")
        .unwrap();
    assert_eq!(
        multi.events,
        vec![
            TriggerEvent::Insert,
            TriggerEvent::Update,
            TriggerEvent::Delete
        ]
    );
    assert_eq!(
        multi.update_columns,
        vec!["Order".to_string(), "a".to_string()]
    );
    assert_eq!(multi.function, "fn_multi");
    assert!(multi.for_each_row);

    let bare = order
        .triggers
        .iter()
        .find(|trigger| trigger.name == "bare")
        .unwrap();
    assert!(!bare.for_each_row);
    assert_eq!(bare.events, vec![TriggerEvent::Insert]);
    let escaped = order
        .triggers
        .iter()
        .find(|trigger| trigger.name == "escaped_ident")
        .unwrap();
    assert_eq!(
        escaped.update_columns,
        vec!["a\"b".to_string(), "fn$1".to_string()]
    );
    assert_eq!(escaped.function, "fn\"x");

    let expr = &catalog
        .table("events_expr")
        .unwrap()
        .partition_key
        .as_ref()
        .unwrap();
    assert_eq!(expr.strategy, PartitionStrategy::Range);
    assert_eq!(
        expr.elements,
        vec![
            PartitionKeyElement::Expression("date_trunc('day'::text, created_at)".to_string()),
            PartitionKeyElement::Column("Order".to_string()),
        ]
    );
    let call = &catalog
        .table("events_call")
        .unwrap()
        .partition_key
        .as_ref()
        .unwrap()
        .elements;
    assert_eq!(
        call.as_slice(),
        [PartitionKeyElement::Expression(
            "coalesce('it''s', 'a,b')".to_string()
        )]
    );
    assert_eq!(
        catalog
            .table("listed")
            .unwrap()
            .partition_key
            .as_ref()
            .unwrap()
            .strategy,
        PartitionStrategy::List
    );
    assert_eq!(
        catalog
            .table("hashed")
            .unwrap()
            .partition_key
            .as_ref()
            .unwrap()
            .elements,
        vec![PartitionKeyElement::Column("id".to_string())]
    );
    assert_eq!(
        catalog
            .table("multi_key")
            .unwrap()
            .partition_key
            .as_ref()
            .unwrap()
            .elements,
        vec![
            PartitionKeyElement::Column("tenant_id".to_string()),
            PartitionKeyElement::Column("id".to_string()),
        ]
    );
    assert_eq!(
        catalog
            .table("quoted_key")
            .unwrap()
            .partition_key
            .as_ref()
            .unwrap()
            .elements,
        vec![PartitionKeyElement::Column("a\"b".to_string())]
    );
    assert_eq!(
        catalog
            .table("paren_expr")
            .unwrap()
            .partition_key
            .as_ref()
            .unwrap()
            .elements,
        vec![PartitionKeyElement::Expression("(created_at)".to_string())]
    );

    let functions = catalog
        .functions()
        .map(|function| (function.key.as_str(), function))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert!(!functions["fn_plain"].returns_trigger);
    assert_eq!(functions["fn_plain"].language.as_deref(), Some("plpgsql"));
    assert!(!functions["fn_setof"].returns_trigger);
    assert_eq!(functions["fn_setof"].body.as_deref(), Some(" SELECT 1 "));
    assert!(functions["fn_no_body"].returns_trigger);
    assert_eq!(functions["fn_no_body"].body, None);
    assert_eq!(
        functions["fn_no_body"].language.as_deref(),
        Some("internal")
    );
    assert_eq!(functions["fn_trigger(a uuid, b text)"].name, "fn_trigger");
    assert_eq!(
        functions["fn_trigger(a uuid, b text)"].signature.as_deref(),
        Some("a uuid, b text")
    );
    assert!(functions["fn_trigger(a uuid, b text)"].returns_trigger);
    assert_eq!(functions["fn_cast"].body.as_deref(), Some(" SELECT 1 "));
    assert!(!functions["fn_word"].returns_trigger);
    assert_eq!(functions["fn_unclosed"].body, None);
    assert_eq!(functions["fn_bare_language"].language, None);
    assert!(!functions["returns_trigger_name"].returns_trigger);
    assert_eq!(functions["fn_open(uuid"].name, "fn_open");
    assert_eq!(functions["fn_open(uuid"].signature.as_deref(), Some("uuid"));
    assert_eq!(functions["fn_open(uuid"].body, None);
    assert_eq!(functions["fn_head_as"].body.as_deref(), Some("z"));
    assert_eq!(
        functions["fn_head_language"].language.as_deref(),
        Some("sql")
    );
    assert!(!functions["fn_returns_only"].returns_trigger);
    assert!(!functions["fn_trigger_word"].returns_trigger);
    assert!(functions["fn_returns_at_start"].returns_trigger);
    assert_eq!(functions["fn_aside"].language, None);
}
