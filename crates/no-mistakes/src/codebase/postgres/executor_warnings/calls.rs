use super::ExecutorWarningScan;
use crate::codebase::ts_source::facts::CallSiteFact;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashMap;
use std::collections::BTreeMap;

pub(super) fn omitted_calls(
    module: &TypeScriptModuleFacts,
    calls: &[CallSiteFact],
    scan: &ExecutorWarningScan,
) -> BTreeMap<String, u32> {
    let bindings = module
        .bindings
        .iter()
        .map(|binding| (binding.id, binding))
        .collect::<FxHashMap<_, _>>();
    let mut references = BTreeMap::new();
    for import in &module.imports {
        if import.type_only || import.specifier != scan.import_specifier {
            continue;
        }
        for imported in &import.bindings {
            if imported.type_only {
                continue;
            }
            let targets = if imported.kind == "named" && scan.omitted.contains(&imported.imported) {
                vec![(imported.local.clone(), imported.imported.clone())]
            } else if imported.kind == "namespace" {
                scan.omitted
                    .iter()
                    .map(|name| (format!("{}.{name}", imported.local), name.clone()))
                    .collect()
            } else {
                continue;
            };
            let Some(binding) = bindings.get(&imported.binding_id) else {
                continue;
            };
            for reference in binding
                .references
                .iter()
                .filter(|reference| reference.runtime)
            {
                references.insert(reference.span.start, targets.clone());
            }
        }
    }
    let mut found = BTreeMap::new();
    for call in calls {
        let Some(targets) = call
            .callee_reference_offset
            .and_then(|offset| references.get(&offset))
        else {
            continue;
        };
        for (callee, name) in targets {
            if &call.callee == callee {
                found
                    .entry(name.clone())
                    .and_modify(|line: &mut u32| *line = (*line).min(call.line))
                    .or_insert(call.line);
            }
        }
    }
    found
}
