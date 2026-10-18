#[test]
fn scalar_and_object_reference_containers_survive_callback_frames_and_slot_joins() {
    let values = outputs_file("reference-fragment-ownership", "query.cjs");
    for (marker, text) in [
        ("refreshed comparison bind", "sql_placeholder_1"),
        ("mutation through retained containers", "SELECT 10"),
        ("retained comparison callback", "SELECT 9"),
        ("callback comparison bind", "sql_placeholder_1"),
        ("callback selected object bind", "sql_placeholder_1"),
        ("joined references retain builder mutation", "SELECT 11"),
        (
            "joined scalar containers remain binds",
            "sql_placeholder_1 sql_placeholder_2",
        ),
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(sql, true, _) if crate::codebase::postgres::embedded::publish_placeholders_with_positions(sql.clone()).0 == text),
            "{marker}"
        );
    }
    assert!(
        matches!(event(&values, "callback object fragment"), Value::Prefix(text, false, _) if text.is_empty())
    );
    assert!(
        matches!(event(&values, "joined object fragment"), Value::Prefix(text, false, _) if text.is_empty())
    );
    let Value::Aggregate(candidates) = event(&values, "possible slot property") else {
        panic!("a possibly replaced object property cannot prove a SQL prefix");
    };
    assert!(candidates.iter().any(|candidate| matches!(candidate,
        Value::Aggregate(properties) if properties.iter().any(|value|
            matches!(value, Value::Prefix(text, true, _) if text == "SELECT 11")
        )
    )));
    assert_eq!(candidates.len(), 2, "both property-read paths must survive");
    assert!(matches!(
        event(&values, "joined comparison escape"),
        Value::Unknown
    ));
    assert_eq!(
        values
            .iter()
            .filter(|(sql, _)| sql.contains("retained comparison callback"))
            .count(),
        1
    );
}
