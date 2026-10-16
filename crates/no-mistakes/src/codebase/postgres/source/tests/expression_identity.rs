use super::{identity, Fields};
use crate::codebase::postgres::source::tests::fixture;

#[test]
fn ordered_identity_retains_raw_scalars_and_legacy_span_swap_removal() {
    let input = fixture("expression-identity-order.json");
    let raw: &serde_json::value::RawValue = serde_json::from_str(&input).unwrap();
    assert_eq!(
        identity(&raw),
        fixture("expression-identity-order.expected.json").trim()
    );
}

#[test]
fn ordered_fields_reject_non_object_input() {
    let input = fixture("expression-identity-non-object.json");
    let error = serde_json::from_str::<Fields<'_>>(&input).err().unwrap();
    assert!(error.to_string().contains("SQL expression object"));
}
