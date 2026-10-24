use super::*;
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};

// An alias is still unsupported. Retain only enough syntax to report Unknown at
// its uses, where line suppression applies, rather than at its declaration.
pub(super) fn collect(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    selections: &[Selection],
    syntax: &mut Syntax,
) {
    let mut sources = FxHashMap::default();
    let mut bindings = FxHashMap::default();
    for binding in &module.bindings {
        bindings.insert((binding.span.start, binding.span.end), binding);
        if selections.iter().any(|selection| {
            selection.selected.contains(&binding.id) || selection.typed.contains(&binding.id)
        }) {
            for usage in &binding.references {
                sources.insert((usage.span.start, usage.span.end), binding.id);
            }
        }
    }
    if sources.is_empty() {
        return;
    }
    Aliases {
        sources,
        bindings,
        syntax,
    }
    .visit_program(program);
}

struct Aliases<'a> {
    sources: FxHashMap<Span, usize>,
    bindings: FxHashMap<Span, &'a ModuleBinding>,
    syntax: &'a mut Syntax,
}

impl<'a> Visit<'a> for Aliases<'_> {
    fn visit_variable_declaration(&mut self, value: &VariableDeclaration<'a>) {
        if value.kind.is_const() {
            for declaration in &value.declarations {
                let Some((source, parts)) = declaration.init.as_ref().and_then(chain) else {
                    continue;
                };
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
                        if property.key.static_name().as_deref() != Some("query") {
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
                        .aliases
                        .insert((usage.span.start, usage.span.end), *owner);
                }
            }
        }
        walk::walk_variable_declaration(self, value);
    }
}
