//! One-hop named helper imports from already prepared source files.
use super::{summary_proof, Bindings, HeadProof, Span, Syntax};
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(crate) struct Pending {
    pub span: Span,
    pub specifier: String,
    pub imported: String,
}

pub(crate) fn module_projection(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: Option<&TypeScriptModuleFacts>,
) -> (BTreeMap<String, HeadProof>, Vec<Pending>) {
    let Some(module) = module else {
        return Default::default();
    };
    let roots = module
        .scopes
        .iter()
        .filter(|scope| scope.parent_id.is_none())
        .map(|scope| scope.id)
        .collect::<FxHashSet<_>>();
    let mut locals: FxHashMap<&str, Option<&ModuleBinding>> = FxHashMap::default();
    for binding in module
        .bindings
        .iter()
        .filter(|binding| binding.runtime && !binding.imported && roots.contains(&binding.scope_id))
    {
        locals
            .entry(binding.name.as_str())
            .and_modify(|owner| *owner = None)
            .or_insert(Some(binding));
    }
    let mut imports: FxHashMap<usize, Option<(&str, &str)>> = FxHashMap::default();
    for import in module.imports.iter().filter(|import| !import.type_only) {
        for binding in import
            .bindings
            .iter()
            .filter(|binding| !binding.type_only && binding.imported != "*")
        {
            imports
                .entry(binding.binding_id)
                .and_modify(|value| *value = None)
                .or_insert(Some((import.specifier.as_str(), binding.imported.as_str())));
        }
    }
    let mut exported = BTreeMap::new();
    let mut seen = FxHashSet::default();
    let mut ambiguous = FxHashSet::default();
    for export in module
        .exports
        .iter()
        .filter(|export| !export.type_only && export.specifier.is_none())
    {
        if !seen.insert(export.exported.as_str()) {
            exported.remove(export.exported.as_str());
            ambiguous.insert(export.exported.as_str());
        }
        if ambiguous.contains(export.exported.as_str()) {
            continue;
        }
        let Some(owner) = locals.get(export.local.as_str()).copied().flatten() else {
            continue;
        };
        let Some(proof) = summary_proof(syntax, bindings, owner) else {
            continue;
        };
        exported.insert(export.exported.clone(), proof);
    }
    let mut pending = Vec::new();
    for (&span, &reference) in &syntax.helper_arguments {
        let Some(owner) = bindings.owner(reference).filter(|owner| owner.imported) else {
            continue;
        };
        let Some((specifier, imported)) = imports.get(&owner.id).copied().flatten() else {
            continue;
        };
        pending.push(Pending {
            span,
            specifier: specifier.to_string(),
            imported: imported.to_string(),
        });
    }
    pending.sort_unstable_by_key(|value| value.span);
    (exported, pending)
}
