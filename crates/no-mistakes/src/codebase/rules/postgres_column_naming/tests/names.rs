use std::collections::BTreeMap;

#[test]
fn singularizer_applies_overrides_then_suffix_rules() {
    let mut overrides = BTreeMap::new();
    overrides.insert("people".to_string(), "person".to_string());
    overrides.insert("aliases".to_string(), "alias".to_string());
    overrides.insert("statuses".to_string(), "status".to_string());
    let singular = |token: &str| super::super::singular::singularize(token, &overrides);
    assert_eq!(singular("items"), "item");
    assert_eq!(singular("currencies"), "currency");
    assert_eq!(singular("addresses"), "address");
    assert_eq!(singular("boxes"), "box");
    assert_eq!(singular("batches"), "batch");
    assert_eq!(singular("dishes"), "dish");
    assert_eq!(singular("staff"), "staff");
    assert_eq!(singular("people"), "person");
    assert_eq!(singular("People"), "person");
    assert_eq!(singular("aliases"), "alias");
    assert_eq!(singular("statuses"), "status");
    assert_eq!(
        super::super::singular::singular_name("order_line_items", &overrides),
        "order_line_item"
    );
    assert_eq!(
        super::super::singular::singular_name("foo__bars", &overrides),
        "foo__bar"
    );
}
