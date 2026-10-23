use super::*;
use std::borrow::Cow;

#[test]
fn mirrored_complete_singleton_keeps_legacy_presentation_and_variant_origins() {
    let file = file_facts("variants-static-append.ts");
    let call = file
        .calls
        .iter()
        .find(|call| call.mirrors_single_variant())
        .expect("saved fixture contains a raw SQL mirror");
    let variant = &call.variants[0];
    assert!(!variant.append_sites.is_empty());
    assert!(!variant.sql_source_offsets.is_empty());

    let projected = call.statement_calls().next().unwrap();
    let Cow::Borrowed(projected) = projected else {
        panic!("mirrored singleton should retain the raw call projection")
    };
    assert_eq!(projected.line, call.line);
    assert_eq!(projected.kind, call.kind);
    assert_eq!(projected.declaration_line, call.declaration_line);
    assert_eq!(projected.sql_source_positions, call.sql_source_positions);
    assert_eq!(
        projected.recovered_placeholder_positions,
        call.recovered_placeholder_positions
    );

    let alternatives = calls("variants-conditional.ts");
    let call = alternatives
        .iter()
        .find(|call| call.variants.len() > 1)
        .expect("saved fixture contains multiple alternatives");
    assert!(!call.mirrors_single_variant());
    assert!(matches!(call.statement_calls().next(), Some(Cow::Owned(_))));
}
