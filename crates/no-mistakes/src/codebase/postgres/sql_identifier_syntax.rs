//! Low-effort SQL inputs from the request's parsed program and semantic bindings.
//! This deliberately does not evaluate JavaScript or follow executor/value aliases.
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
#[derive(Default)]
pub(crate) struct FileFacts {
    pub profiles: Vec<(EmbeddedSqlOptions, Vec<SqlCall>)>,
}
struct Call {
    span: Span,
    callee: Option<(Span, Vec<String>)>,
    input: Input,
}
#[derive(Default)]
struct Syntax {
    calls: Vec<Call>,
    constants: FxHashMap<Span, Input>,
    env_references: FxHashMap<String, Span>,
    namespace_members: FxHashMap<Span, String>,
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
fn input(expression: &Expression<'_>, env: &mut FxHashMap<String, Span>) -> Input {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(value) => Input::Sql {
            text: value.value.to_string(),
            holes: Vec::new(),
        },
        Expression::TemplateLiteral(value) => {
            let mut text = String::new();
            let mut holes = Vec::new();
            let mut prefix = "no_mistakes_low_hole_".to_string();
            while value.quasis.iter().any(|part| {
                part.value
                    .cooked
                    .as_ref()
                    .is_some_and(|text| text.contains(&prefix))
            }) {
                prefix.push('_');
            }
            for (index, quasi) in value.quasis.iter().enumerate() {
                let Some(cooked) = &quasi.value.cooked else {
                    return Input::Unknown;
                };
                text.push_str(cooked);
                if let Some(expression) = value.expressions.get(index) {
                    // Placeholder identity preserves which SQL position owns each input.
                    let placeholder = format!("{prefix}{index}");
                    text.push_str(&placeholder);
                    let source = chain(expression).and_then(|(root, parts)| {
                        (parts.len() == 3 && parts[0] == "process" && parts[1] == "env").then(
                            || {
                                let key = format!("process.env.{}", parts[2]);
                                env.insert(format!("{key}@{}", root.0), root);
                                format!("{key}@{}", root.0)
                            },
                        )
                    });
                    holes.push((placeholder, source));
                }
            }
            Input::Sql { text, holes }
        }
        Expression::Identifier(id) => Input::Reference(span(id.span)),
        _ => Input::Unknown,
    }
}
impl<'a> Visit<'a> for Syntax {
    fn visit_static_member_expression(&mut self, value: &oxc_ast::ast::StaticMemberExpression<'a>) {
        if let Some((root, parts)) = chain(&value.object) {
            if parts.len() == 1 {
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
            if parts.len() == 1 {
                if let Expression::StringLiteral(key) = unwrap_ts_wrappers(&value.expression) {
                    self.namespace_members.insert(root, key.value.to_string());
                }
            }
        }
        walk::walk_computed_member_expression(self, value);
    }
    fn visit_call_expression(&mut self, value: &CallExpression<'a>) {
        self.calls.push(Call {
            span: span(value.span),
            callee: chain(&value.callee),
            input: value
                .arguments
                .first()
                .and_then(|arg| arg.as_expression())
                .map_or(Input::Unknown, |value| {
                    input(value, &mut self.env_references)
                }),
        });
        walk::walk_call_expression(self, value);
    }
    fn visit_variable_declaration(&mut self, value: &VariableDeclaration<'a>) {
        if value.kind.is_const() {
            for declaration in &value.declarations {
                if let BindingPattern::BindingIdentifier(binding) = &declaration.id {
                    let value = declaration.init.as_ref().map_or(Input::Unknown, |value| {
                        input(value, &mut self.env_references)
                    });
                    if matches!(value, Input::Sql { .. }) {
                        self.constants.insert(span(binding.span), value);
                    }
                }
            }
        }
        walk::walk_variable_declaration(self, value);
    }
}
mod project;
pub(crate) use project::collect;
