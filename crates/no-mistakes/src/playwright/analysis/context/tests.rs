use super::resolve_test_id_attributes;

fn ids(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

#[test]
fn override_takes_highest_precedence() {
    let attributes =
        resolve_test_id_attributes(Some("data-qa"), Some("data-pw"), &ids(&["data-foo"]));
    assert_eq!(attributes, ids(&["data-pw"]));
}

#[test]
fn readable_project_attribute_wins_over_configured_test_ids() {
    let attributes = resolve_test_id_attributes(Some("data-qa"), None, &ids(&["data-pw"]));
    assert_eq!(attributes, ids(&["data-qa"]));
}

#[test]
fn falls_back_to_sorted_deduped_configured_test_ids() {
    let attributes =
        resolve_test_id_attributes(None, None, &ids(&["data-pw", "data-testid", "data-pw"]));
    assert_eq!(attributes, ids(&["data-pw", "data-testid"]));
}

#[test]
fn falls_back_to_default_when_nothing_configured() {
    let attributes = resolve_test_id_attributes(None, None, &[]);
    assert_eq!(
        attributes,
        ids(&[crate::playwright::playwright_config::DEFAULT_TEST_ID_ATTRIBUTE])
    );
}
