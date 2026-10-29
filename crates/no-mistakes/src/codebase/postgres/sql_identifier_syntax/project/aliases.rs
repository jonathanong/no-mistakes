use super::*;
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
pub(super) mod detached;
impl Syntax<'_> {
    pub(in crate::codebase::postgres::sql_identifier_syntax) fn selected_callee(
        &self,
        root: &Span,
        parts: &[String],
    ) -> bool {
        (parts.len() == 1 && self.alias_target(root))
            || (parts.len() == 2 && self.aliases.contains_key(root) && parts[1] == "query")
            || (self.typed_targets.contains(root) && input::indirect_executor_call(parts))
            || (parts.len() == 2
                && self
                    .namespace_targets
                    .get(root)
                    .is_some_and(|names| names.contains(&parts[1])))
            || (self.loose_query
                && parts.len() > 1
                && parts.last().is_some_and(|name| name == "query"))
    }
    pub(in crate::codebase::postgres::sql_identifier_syntax) fn alias_target(
        &self,
        at: &Span,
    ) -> bool {
        self.targets.contains(at) || self.aliases.contains_key(at)
    }
}

// An alias is still unsupported. Retain only enough syntax to report Unknown at
// its uses, where line suppression applies, rather than at its declaration.
pub(super) fn collect(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    selections: &mut [Selection],
    syntax: &mut Syntax,
) {
    let advisory = selections
        .iter()
        .any(|selection| !selection.advisory_receivers.is_empty());
    let mut sources = FxHashMap::default();
    let mut references = FxHashMap::default();
    let mut bindings = FxHashMap::default();
    for binding in &module.bindings {
        bindings.insert((binding.span.start, binding.span.end), binding);
        if advisory {
            for usage in binding.references.iter().filter(|usage| usage.runtime) {
                references.insert((usage.span.start, usage.span.end), binding.id);
            }
        }
        if selections.iter().any(|selection| {
            selection.selected.contains(&binding.id) || selection.typed.contains(&binding.id)
        }) {
            for usage in &binding.references {
                sources.insert((usage.span.start, usage.span.end), binding.id);
            }
        }
    }
    if sources.is_empty() && !advisory {
        return;
    }
    let mut aliases = Aliases {
        sources,
        references,
        bindings,
        syntax,
        advisory_edges: FxHashMap::default(),
    };
    aliases.visit_program(program);
    for selection in selections {
        let mut pending = selection
            .advisory_receivers
            .iter()
            .copied()
            .collect::<Vec<_>>();
        while let Some(source) = pending.pop() {
            for alias in aliases.advisory_edges.get(&source).into_iter().flatten() {
                if selection.advisory_receivers.insert(*alias) {
                    pending.push(*alias);
                }
            }
        }
    }
}

struct Aliases<'a, 'source> {
    sources: FxHashMap<Span, usize>,
    references: FxHashMap<Span, usize>,
    bindings: FxHashMap<Span, &'a ModuleBinding>,
    syntax: &'a mut Syntax<'source>,
    advisory_edges: FxHashMap<usize, Vec<usize>>,
}

impl Aliases<'_, '_> {
    fn advisory_edge(&mut self, source: Span, alias: usize) {
        if let Some(source_id) = self.references.get(&source) {
            self.advisory_edges
                .entry(*source_id)
                .or_default()
                .push(alias);
        }
    }
}

impl<'a> Visit<'a> for Aliases<'_, '_> {
    fn visit_variable_declaration(&mut self, value: &VariableDeclaration<'a>) {
        for declaration in &value.declarations {
            let Some((source, parts)) = declaration.init.as_ref().and_then(chain) else {
                continue;
            };
            if parts.len() == 1 {
                if let BindingPattern::BindingIdentifier(alias) = &declaration.id {
                    if let Some(id) = self
                        .bindings
                        .get(&span(alias.span))
                        .map(|binding| binding.id)
                    {
                        self.advisory_edge(source, id);
                    }
                }
            }
            if value.kind.is_const() {
                let alias = match &declaration.id {
                    BindingPattern::BindingIdentifier(alias)
                        if parts.len() == 1 || (parts.len() == 2 && parts[1] == "query") =>
                    {
                        alias
                    }
                    BindingPattern::ObjectPattern(object)
                        if parts.len() == 1
                            && object.properties.len() == 1
                            && object.rest.is_none() =>
                    {
                        let property = &object.properties[0];
                        if crate::ast::property_key_name(&property.key).as_deref() != Some("query")
                        {
                            continue;
                        }
                        let BindingPattern::BindingIdentifier(alias) = &property.value else {
                            continue;
                        };
                        alias
                    }
                    _ => continue,
                };
                let Some(owner) = self.sources.get(&source) else {
                    continue;
                };
                let Some(binding) = self.bindings.get(&span(alias.span)) else {
                    continue;
                };
                self.syntax.alias_initializers.insert(source, *owner);
                for usage in binding.references.iter().filter(|usage| usage.runtime) {
                    self.syntax
                        .metadata
                        .track_alias((usage.span.start, usage.span.end));
                    self.syntax
                        .aliases
                        .insert((usage.span.start, usage.span.end), *owner);
                }
            }
        }
        walk::walk_variable_declaration(self, value);
    }
}
