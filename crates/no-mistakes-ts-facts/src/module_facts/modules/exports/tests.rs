use super::*;
#[test]
fn inline_exports_inspect_only_indexed_declaration_bindings() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/typescript-module-facts/many-exports.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        crate::module_facts::extract_with_arguments(program, false)
    })
    .unwrap();
    assert_eq!(facts.exports.len(), 67);
    assert!(!facts.exports.iter().any(|export| export.local == "nested"));
    let mut indices = facts
        .bindings
        .iter()
        .enumerate()
        .filter(|(_, binding)| binding.scope_id == 0)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    indices.sort_unstable_by_key(|&index| facts.bindings[index].span.start);
    let mut inspected = 0;
    for export in &facts.exports {
        inspected += inline_candidates(
            &indices,
            &facts.bindings,
            oxc_span::Span::new(export.span.start, export.span.end),
        )
        .len();
    }
    // Lock bounded selection work, not just output equality: scanning every
    // binding for every export would inspect thousands of entries here.
    assert_eq!(inspected, 67);
}
