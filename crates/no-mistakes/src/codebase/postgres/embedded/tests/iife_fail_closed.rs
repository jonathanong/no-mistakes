use super::*;
use crate::codebase::postgres::embedded::EmbeddedSqlKind;
use std::path::PathBuf;

#[test]
fn imported_iife_shapes_that_are_not_safe_stay_opaque() {
    let facts = extract_from_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-required-predicates/variants/imported-iife-fail-closed/src/caller.mts",
    ));
    assert!(!facts.calls.is_empty());
    for call in &facts.calls {
        assert_eq!(call.kind, EmbeddedSqlKind::Dynamic, "{call:#?}");
        assert!(
            call.variants.is_empty(),
            "unsafe IIFE must not publish recovered alternatives: {call:#?}"
        );
        assert!(
            call.sql_text.is_none(),
            "unsafe IIFE must keep the executor argument opaque: {call:#?}"
        );
    }
}
