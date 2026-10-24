//! Low-effort SQL inputs from the request's parsed program and semantic bindings.
//! This does not evaluate JavaScript or infer provenance through executor/value aliases.
pub(crate) mod literal_projection;
use super::EmbeddedSqlOptions;
use crate::codebase::ts_source::{facts::TsFileFacts, unwrap_ts_wrappers};
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{BindingPattern, CallExpression, Expression, Program, VariableDeclaration};
use oxc_ast_visit::{walk, Visit};

type Span = (u32, u32);
fn span(value: oxc_span::Span) -> Span {
    (value.start, value.end)
}

#[derive(Clone, Debug)]
pub(crate) enum Input {
    Sql {
        text: String,
        source_span: Span,
        holes: Vec<(String, Option<String>)>,
    },
    Reference(Span),
    Unknown,
}
#[derive(Clone, Debug)]
pub(crate) struct SqlCall {
    pub span: Span,
    pub line: u32,
    pub input: Input,
}
pub(crate) type OffsetLiteral = std::result::Result<Option<Vec<super::SqlOffsetFact>>, ()>;
pub(crate) type TransformedLiteral =
    std::result::Result<Option<Vec<super::statements::SqlTransformedColumnUseFact>>, ()>;
#[derive(Default)]
pub(crate) struct FileFacts {
    pub profiles: Vec<(EmbeddedSqlOptions, Vec<SqlCall>)>,
    pub transformed_literals:
        std::sync::OnceLock<std::collections::BTreeMap<Span, TransformedLiteral>>,
    pub offset_literals: std::sync::OnceLock<std::collections::BTreeMap<Span, OffsetLiteral>>,
}
struct Call {
    span: Span,
    callee: Option<(Span, Vec<String>)>,
    input: Input,
}
#[derive(Default)]
struct Syntax {
    targets: FxHashSet<Span>,
    namespace_targets: FxHashMap<Span, FxHashSet<String>>,
    loose_query: bool,
    calls: Vec<Call>,
    constants: FxHashMap<Span, Input>,
    env_references: FxHashMap<String, Span>,
    namespace_members: FxHashMap<Span, String>,
    aliases: FxHashMap<Span, usize>,
    alias_initializers: FxHashMap<Span, usize>,
}
fn chain(expression: &Expression<'_>) -> Option<(Span, Vec<String>)> {
    match unwrap_ts_wrappers(expression) {
        Expression::Identifier(id) => Some((span(id.span), vec![id.name.to_string()])),
        Expression::StaticMemberExpression(value) => {
            let (root, mut parts) = chain(&value.object)?;
            parts.push(value.property.name.to_string());
            Some((root, parts))
        }
        Expression::ComputedMemberExpression(value) => {
            let Expression::StringLiteral(key) = unwrap_ts_wrappers(&value.expression) else {
                return None;
            };
            let (root, mut parts) = chain(&value.object)?;
            parts.push(key.value.to_string());
            Some((root, parts))
        }
        _ => None,
    }
}
mod input;
use input::input;
impl<'a> Visit<'a> for Syntax {
    fn visit_static_member_expression(&mut self, value: &oxc_ast::ast::StaticMemberExpression<'a>) {
        if let Some((root, parts)) = chain(&value.object) {
            if parts.len() == 1 && self.namespace_targets.contains_key(&root) {
                self.namespace_members
                    .insert(root, value.property.name.to_string());
            }
        }
        walk::walk_static_member_expression(self, value);
    }
    fn visit_computed_member_expression(
        &mut self,
        value: &oxc_ast::ast::ComputedMemberExpression<'a>,
    ) {
        if let Some((root, parts)) = chain(&value.object) {
            if parts.len() == 1 && self.namespace_targets.contains_key(&root) {
                if let Expression::StringLiteral(key) = unwrap_ts_wrappers(&value.expression) {
                    self.namespace_members.insert(root, key.value.to_string());
                }
            }
        }
        walk::walk_computed_member_expression(self, value);
    }
    fn visit_call_expression(&mut self, value: &CallExpression<'a>) {
        let callee = chain(&value.callee);
        let selected = callee.as_ref().is_some_and(|(root, parts)| {
            (parts.len() == 1 && (self.targets.contains(root) || self.aliases.contains_key(root)))
                || (parts.len() == 2
                    && self
                        .namespace_targets
                        .get(root)
                        .is_some_and(|names| names.contains(&parts[1])))
                || (self.loose_query
                    && parts.len() > 1
                    && parts.last().is_some_and(|name| name == "query"))
        });
        if selected {
            self.calls.push(Call {
                span: span(value.span),
                input: if callee
                    .as_ref()
                    .is_some_and(|(root, parts)| parts.len() == 1 && self.targets.contains(root))
                {
                    value
                        .arguments
                        .first()
                        .and_then(|arg| arg.as_expression())
                        .map_or(Input::Unknown, |value| {
                            input(value, &mut self.env_references)
                        })
                } else {
                    Input::Unknown
                },
                callee,
            });
        }
        walk::walk_call_expression(self, value);
    }
}
struct Constants<'s> {
    wanted: &'s FxHashSet<Span>,
    syntax: &'s mut Syntax,
}
impl<'a> Visit<'a> for Constants<'_> {
    fn visit_variable_declaration(&mut self, value: &VariableDeclaration<'a>) {
        if value.kind.is_const() {
            for declaration in &value.declarations {
                if let BindingPattern::BindingIdentifier(binding) = &declaration.id {
                    if !self.wanted.contains(&span(binding.span)) {
                        continue;
                    }
                    let value = declaration.init.as_ref().map_or(Input::Unknown, |value| {
                        input(value, &mut self.syntax.env_references)
                    });
                    if matches!(value, Input::Sql { .. }) {
                        self.syntax.constants.insert(span(binding.span), value);
                    }
                }
            }
        }
        walk::walk_variable_declaration(self, value);
    }
}
mod selection;
use selection::Selection;
mod project;
pub(crate) use project::collect;

mod invalid;
pub(crate) use invalid::collect_invalid;

pub(crate) mod plan;

#[cfg(test)]
mod tests;
