use super::HeadProof;
use crate::codebase::postgres::TrustedSqlTag;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::sync::Arc;
pub(super) fn identities(
    module: &TypeScriptModuleFacts,
    wanted: &FxHashSet<usize>,
) -> FxHashMap<usize, HeadProof> {
    let mut identities = FxHashMap::default();
    for import in &module.imports {
        for binding in &import.bindings {
            if !wanted.contains(&binding.binding_id) {
                continue;
            }
            let proof = if !import.type_only && !binding.type_only && binding.kind == "named" {
                HeadProof::NamedImport(Arc::new(TrustedSqlTag {
                    module: import.specifier.clone(),
                    name: binding.imported.clone(),
                }))
            } else {
                HeadProof::Unproven
            };
            identities
                .entry(binding.binding_id)
                .and_modify(|entry| *entry = HeadProof::Unproven)
                .or_insert(proof);
        }
    }
    identities
}
