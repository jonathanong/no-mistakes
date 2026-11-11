use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashMap;
use oxc_semantic::Semantic;
mod call;
mod safety;

#[derive(Clone)]
pub(in crate::codebase::ts_source::module_facts::arguments) enum TempFactoryBinding {
    Named(bool),
    Module { sync: bool, promises_safe: bool },
}

fn node_fs(specifier: &str) -> bool {
    matches!(
        specifier,
        "node:fs" | "fs" | "node:fs/promises" | "fs/promises"
    )
}

fn factory(name: &str, specifier: &str) -> bool {
    match name {
        "mkdtemp" => node_fs(specifier),
        "mkdtempSync" => matches!(specifier, "node:fs" | "fs"),
        _ => false,
    }
}

pub(in crate::codebase::ts_source::module_facts::arguments) fn temp_factories(
    semantic: &Semantic<'_>,
    facts: &TypeScriptModuleFacts,
) -> FxHashMap<usize, TempFactoryBinding> {
    let mut bindings = FxHashMap::default();
    let loads: FxHashMap<_, _> = facts
        .loads
        .iter()
        .map(|load| (load.span.start, load.specifier.as_str()))
        .collect();
    if !facts.imports.iter().any(|import| {
        !import.type_only
            && import.bindings.iter().any(|binding| {
                !binding.type_only
                    && (factory(&binding.imported, &import.specifier)
                        || binding.kind == "named"
                            && binding.imported == "promises"
                            && matches!(import.specifier.as_str(), "fs" | "node:fs")
                        || node_fs(&import.specifier)
                            && matches!(binding.kind.as_str(), "namespace" | "default"))
            })
    }) && !facts.require_bindings.iter().any(|binding| {
        loads.get(&binding.load_start).is_some_and(|specifier| {
            binding.imported.as_deref().map_or_else(
                || node_fs(specifier),
                |name| {
                    factory(name, specifier)
                        || name == "promises" && matches!(*specifier, "fs" | "node:fs")
                },
            )
        })
    }) {
        return bindings;
    }
    let symbols: FxHashMap<_, _> = semantic
        .scoping()
        .symbol_ids()
        .map(|symbol| (symbol.index(), symbol))
        .collect();
    let locals: FxHashMap<_, _> = facts
        .bindings
        .iter()
        .map(|binding| ((binding.scope_id, binding.name.as_str()), binding.id))
        .collect();
    let mut unsafe_sync = false;
    let mut unsafe_promises = false;
    for import in &facts.imports {
        if import.type_only || !node_fs(&import.specifier) {
            continue;
        }
        let sync = matches!(import.specifier.as_str(), "fs" | "node:fs");
        for binding in &import.bindings {
            if binding.type_only {
                continue;
            }
            let symbol = symbols.get(&binding.binding_id).copied();
            let promises = binding.kind == "named" && binding.imported == "promises" && sync;
            if binding.kind == "named"
                && factory(&binding.imported, &import.specifier)
                && symbol.is_some_and(|symbol| {
                    semantic
                        .scoping()
                        .get_resolved_references(symbol)
                        .all(|reference| !reference.is_write())
                })
            {
                bindings.insert(binding.binding_id, TempFactoryBinding::Named(sync));
            } else if promises || matches!(binding.kind.as_str(), "namespace" | "default") {
                if let Some(symbol) = symbol {
                    let module_sync = sync && !promises;
                    let unsafe_family = if module_sync {
                        &mut unsafe_sync
                    } else {
                        &mut unsafe_promises
                    };
                    safety::record_module(
                        semantic,
                        symbol,
                        binding.binding_id,
                        module_sync,
                        &mut bindings,
                        unsafe_family,
                    );
                }
            }
        }
    }
    for binding in &facts.require_bindings {
        let Some(&specifier) = loads.get(&binding.load_start) else {
            continue;
        };
        if !node_fs(specifier) {
            continue;
        }
        let sync = matches!(specifier, "fs" | "node:fs");
        let Some(&id) = locals.get(&(binding.scope_id, binding.local.as_str())) else {
            continue;
        };
        if let Some(name) = &binding.imported {
            if factory(name, specifier)
                && symbols.get(&id).copied().is_some_and(|symbol| {
                    semantic
                        .scoping()
                        .get_resolved_references(symbol)
                        .all(|reference| !reference.is_write())
                })
            {
                bindings.insert(id, TempFactoryBinding::Named(sync));
            } else if name == "promises" && sync {
                if let Some(&symbol) = symbols.get(&id) {
                    safety::record_module(
                        semantic,
                        symbol,
                        id,
                        false,
                        &mut bindings,
                        &mut unsafe_promises,
                    );
                }
            }
        } else if let Some(&symbol) = symbols.get(&id) {
            let unsafe_family = if sync {
                &mut unsafe_sync
            } else {
                &mut unsafe_promises
            };
            safety::record_module(semantic, symbol, id, sync, &mut bindings, unsafe_family);
        }
    }
    if unsafe_sync {
        unsafe_promises = true;
    }
    bindings.retain(|_, binding| {
        let sync = match binding {
            TempFactoryBinding::Named(sync) => *sync,
            TempFactoryBinding::Module {
                sync,
                promises_safe,
            } => {
                if *sync {
                    *promises_safe = !unsafe_promises;
                }
                *sync
            }
        };
        if sync {
            !unsafe_sync
        } else {
            !unsafe_promises
        }
    });
    bindings
}
