use super::Projector;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashMap;
use oxc_ast::ast::*;
use oxc_semantic::Semantic;
mod safety;

#[derive(Clone)]
pub(in crate::codebase::ts_source::module_facts::arguments) enum TempFactoryBinding {
    Named(bool),
    Module(bool),
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
                        || node_fs(&import.specifier)
                            && matches!(binding.kind.as_str(), "namespace" | "default"))
            })
    }) && !facts.require_bindings.iter().any(|binding| {
        loads.get(&binding.load_start).is_some_and(|specifier| {
            binding
                .imported
                .as_deref()
                .map_or_else(|| node_fs(specifier), |name| factory(name, specifier))
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
            } else if matches!(binding.kind.as_str(), "namespace" | "default") {
                let stable =
                    symbol.is_some_and(|symbol| safety::factory_view_safe(semantic, symbol));
                if stable {
                    bindings.insert(binding.binding_id, TempFactoryBinding::Module(sync));
                } else if symbol.is_some_and(|symbol| {
                    semantic
                        .scoping()
                        .get_resolved_references(symbol)
                        .next()
                        .is_some()
                }) {
                    if sync {
                        unsafe_sync = true;
                    } else {
                        unsafe_promises = true;
                    }
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
            }
        } else if let Some(&symbol) = symbols.get(&id) {
            if safety::factory_view_safe(semantic, symbol) {
                bindings.insert(id, TempFactoryBinding::Module(sync));
            } else if semantic
                .scoping()
                .get_resolved_references(symbol)
                .next()
                .is_some()
            {
                if sync {
                    unsafe_sync = true;
                } else {
                    unsafe_promises = true;
                }
            }
        }
    }
    bindings.retain(|_, binding| {
        let sync = match binding {
            TempFactoryBinding::Named(sync) | TempFactoryBinding::Module(sync) => *sync,
        };
        if sync {
            !unsafe_sync
        } else {
            !unsafe_promises
        }
    });
    bindings
}

impl Projector<'_, '_> {
    pub(super) fn is_temp_factory(&self, callee: &Expression<'_>) -> bool {
        match callee {
            Expression::Identifier(id) => self
                .binding_symbol(id)
                .and_then(|symbol| self.temp_factories.get(&symbol.index()))
                .is_some_and(|binding| matches!(binding, TempFactoryBinding::Named(_))),
            Expression::StaticMemberExpression(member) => {
                let Expression::Identifier(id) = member.object.get_inner_expression() else {
                    return false;
                };
                self.binding_symbol(id)
                    .and_then(|symbol| self.temp_factories.get(&symbol.index()))
                    .is_some_and(|binding| match binding {
                        TempFactoryBinding::Module(sync) => {
                            member.property.name == "mkdtemp"
                                || *sync && member.property.name == "mkdtempSync"
                        }
                        TempFactoryBinding::Named(_) => false,
                    })
            }
            _ => false,
        }
    }
}
