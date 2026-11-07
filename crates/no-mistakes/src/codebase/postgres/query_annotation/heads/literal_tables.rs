use super::{bindings::Bindings, syntax::Syntax, HeadProof, Span};
use crate::codebase::ts_source::{module_facts::TypeScriptModuleFacts, unwrap_ts_wrappers};
use crate::fx::FxHashMap;
use oxc_ast::ast::*;

#[derive(Default)]
pub(super) struct Tables {
    declarations: FxHashMap<Span, FxHashMap<Vec<String>, HeadProof>>,
    reads: FxHashMap<Span, (Span, Vec<String>)>,
}
impl Tables {
    pub(super) fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }
    pub(super) fn declaration(&mut self, value: &VariableDeclarator<'_>) {
        let BindingPattern::BindingIdentifier(id) = &value.id else {
            return;
        };
        let Some(Expression::TSAsExpression(assertion)) = &value.init else {
            return;
        };
        let TSType::TSTypeReference(reference) = &assertion.type_annotation else {
            return;
        };
        let TSTypeName::IdentifierReference(name) = &reference.type_name else {
            return;
        };
        if name.name != "const" {
            return;
        }
        let mut leaves = FxHashMap::default();
        if collect(&assertion.expression, &mut Vec::new(), &mut leaves, 0).is_some() {
            self.declarations
                .insert((id.span.start, id.span.end), leaves);
        }
    }
    pub(super) fn call(&mut self, value: &CallExpression<'_>) {
        if value.optional {
            return;
        }
        let Some(argument) = value.arguments.first().and_then(Argument::as_expression) else {
            return;
        };
        let mut path = Vec::new();
        let mut expression = unwrap_ts_wrappers(argument);
        while let Expression::StaticMemberExpression(member) = expression {
            if member.optional || path.len() == 8 {
                return;
            }
            path.push(member.property.name.to_string());
            expression = unwrap_ts_wrappers(&member.object);
        }
        if path.is_empty() {
            return;
        }
        if let Expression::Identifier(id) = expression {
            path.reverse();
            self.reads.insert(
                (id.span.start, id.span.end),
                ((value.span.start, value.span.end), path),
            );
        }
    }
}
fn collect(
    expression: &Expression<'_>,
    path: &mut Vec<String>,
    leaves: &mut FxHashMap<Vec<String>, HeadProof>,
    depth: u8,
) -> Option<()> {
    if depth > 8 || leaves.len() >= 256 {
        return None;
    }
    match unwrap_ts_wrappers(expression) {
        Expression::ObjectExpression(object) => {
            let mut keys = crate::fx::FxHashSet::default();
            for property in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    return None;
                };
                if property.computed
                    || property.method
                    || property.shorthand
                    || property.kind != PropertyKind::Init
                {
                    return None;
                }
                let PropertyKey::StaticIdentifier(key) = &property.key else {
                    return None;
                };
                if key.name == "__proto__" || !keys.insert(key.name.as_str()) {
                    return None;
                }
                path.push(key.name.to_string());
                collect(&property.value, path, leaves, depth + 1)?;
                path.pop();
            }
        }
        expression if !path.is_empty() => {
            let text = super::literal_helpers::literal_text(expression)?;
            let proof = if crate::codebase::postgres::sql_requires_query_annotation(&text) {
                HeadProof::Missing
            } else {
                HeadProof::Literal
            };
            leaves.insert(path.clone(), proof);
        }
        _ => return None,
    }
    Some(())
}
pub(super) fn proofs(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: Option<&TypeScriptModuleFacts>,
    executors: &crate::fx::FxHashSet<Span>,
) -> Vec<(Span, HeadProof)> {
    if syntax.literal_tables.declarations.is_empty() {
        return Vec::new();
    }
    let Some(module) = module else {
        return Vec::new();
    };
    let exported = module
        .exports
        .iter()
        .filter(|export| !export.type_only)
        .map(|export| export.local.as_str())
        .collect::<crate::fx::FxHashSet<_>>();
    let mut result = Vec::new();
    for owner in &module.bindings {
        let declaration = (owner.span.start, owner.span.end);
        let Some(leaves) = syntax.literal_tables.declarations.get(&declaration) else {
            continue;
        };
        if owner.imported
            || exported.contains(owner.name.as_str())
            || !bindings.declaration_safe(declaration)
        {
            continue;
        }
        let reads = owner
            .references
            .iter()
            .filter(|usage| usage.runtime)
            .map(|usage| {
                let reference = (usage.span.start, usage.span.end);
                if !bindings.declaration_safe(reference) {
                    return None;
                }
                let (call, path) = syntax.literal_tables.reads.get(&reference)?;
                if !executors.contains(call) {
                    return None;
                }
                Some((*call, *leaves.get(path)?))
            })
            .collect::<Option<Vec<_>>>();
        if let Some(reads) = reads {
            result.extend(reads);
        }
    }
    result
}
