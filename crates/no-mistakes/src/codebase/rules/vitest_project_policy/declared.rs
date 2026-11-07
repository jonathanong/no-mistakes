use super::options::{Declared, LiteralKind};
use super::*;
use crate::codebase::ts_source::{static_property_key_name, unwrap_ts_wrappers};
use oxc_ast::ast::{Expression, ObjectProperty};
use oxc_ast_visit::{walk, Visit};
use rayon::prelude::*;

pub(super) fn check(
    root: &Path,
    options: &Option<Declared>,
    paths: &BTreeSet<PathBuf>,
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    let Some(options) = options else {
        return Vec::new();
    };
    let programs = crate::ast::current_request_parse_cache().unwrap_or_default();
    let declarations = paths
        .iter()
        .flat_map(|path| {
            let Ok(source) = sources.read_path(path) else {
                return Vec::new();
            };
            programs
                .with_program(path, source, |program, source| {
                    let mut visitor = Declarations {
                        path,
                        source,
                        values: Vec::new(),
                    };
                    visitor.visit_program(program);
                    visitor.values
                })
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    declarations.into_par_iter().filter_map(|declaration| {
        let (key, value) = (&declaration.key, &declaration.value);
        let forbidden = options.forbidden_values.get(key)
            .is_some_and(|values| values.iter().any(|banned| value.matches(banned)));
        let kind = options.forbid_literal_kinds.get(key)
            .is_some_and(|kinds| kinds.iter().any(|kind| match kind {
                LiteralKind::Number => matches!(value, Literal::Number(_)),
                LiteralKind::String => matches!(value, Literal::Other(serde_json::Value::String(_))),
                LiteralKind::Boolean => matches!(value, Literal::Other(serde_json::Value::Bool(_))),
            }));
        (forbidden || kind).then(|| RuleFinding {
            rule: RULE_ID.into(), file: relative_slash_path(root, &declaration.path), line: declaration.line,
            message: format!("Vitest declaration {key}: literal {value} is forbidden by {}; use an allowed literal or a computed expression to keep configuration policy explicit",
                if forbidden { "declared.forbiddenValues" } else { "declared.forbidLiteralKinds" }),
            import: None, target: None,
        })
    }).collect()
}

struct Declaration {
    path: PathBuf,
    line: usize,
    key: String,
    value: Literal,
}
struct Declarations<'a> {
    path: &'a Path,
    source: &'a str,
    values: Vec<Declaration>,
}

impl<'a> Visit<'a> for Declarations<'_> {
    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        if !property.computed && !property.method {
            if let (Some(key), Some(value)) = (
                static_property_key_name(&property.key),
                literal(&property.value),
            ) {
                self.values.push(Declaration {
                    path: self.path.to_path_buf(),
                    line: crate::codebase::ts_source::byte_offset_to_line(
                        self.source,
                        property.span.start as usize,
                    ) as usize,
                    key: key.into_owned(),
                    value,
                });
            }
        }
        walk::walk_object_property(self, property);
    }
}

enum Literal {
    Number(f64),
    Other(serde_json::Value),
}
impl Literal {
    fn matches(&self, banned: &serde_json::Value) -> bool {
        match self {
            Self::Number(number) => banned.as_f64() == Some(*number),
            Self::Other(value) => value == banned,
        }
    }
}
impl std::fmt::Display for Literal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(number) => write!(formatter, "{number}"),
            Self::Other(value) => write!(formatter, "{value}"),
        }
    }
}

fn literal(expression: &Expression<'_>) -> Option<Literal> {
    match unwrap_ts_wrappers(expression) {
        Expression::BooleanLiteral(value) => Some(Literal::Other(value.value.into())),
        Expression::NullLiteral(_) => Some(Literal::Other(serde_json::Value::Null)),
        Expression::StringLiteral(value) => Some(Literal::Other(
            crate::ast::js_literal(value).as_ref().into(),
        )),
        Expression::NumericLiteral(value) => Some(Literal::Number(value.value)),
        Expression::UnaryExpression(unary) => {
            let Expression::NumericLiteral(value) = unwrap_ts_wrappers(&unary.argument) else {
                return None;
            };
            let number = match unary.operator {
                oxc_ast::ast::UnaryOperator::UnaryNegation => -value.value,
                oxc_ast::ast::UnaryOperator::UnaryPlus => value.value,
                _ => return None,
            };
            Some(Literal::Number(number))
        }
        _ => None,
    }
}
