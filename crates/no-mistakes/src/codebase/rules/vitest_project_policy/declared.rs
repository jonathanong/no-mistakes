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
    paths
        .par_iter()
        .flat_map(|path| {
            let Ok(source) = sources.read_path(path) else {
                return Vec::new();
            };
            programs
                .with_program(path, source, |program, source| {
                    let mut visitor = Declarations {
                        root,
                        path,
                        source,
                        options,
                        findings: Vec::new(),
                    };
                    visitor.visit_program(program);
                    visitor.findings
                })
                .unwrap_or_default()
        })
        .collect()
}

struct Declarations<'a> {
    root: &'a Path,
    path: &'a Path,
    source: &'a str,
    options: &'a Declared,
    findings: Vec<RuleFinding>,
}

impl<'a> Visit<'a> for Declarations<'_> {
    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        if !property.computed && !property.method {
            if let (Some(key), Some(value)) = (
                static_property_key_name(&property.key),
                literal(&property.value),
            ) {
                let forbidden = self
                    .options
                    .forbidden_values
                    .get(key.as_ref())
                    .is_some_and(|values| values.iter().any(|banned| equal(banned, &value)));
                let kind = self
                    .options
                    .forbid_literal_kinds
                    .get(key.as_ref())
                    .is_some_and(|kinds| {
                        kinds.iter().any(|kind| match kind {
                            LiteralKind::Number => value.is_number(),
                            LiteralKind::String => value.is_string(),
                            LiteralKind::Boolean => value.is_boolean(),
                        })
                    });
                if forbidden || kind {
                    self.findings.push(RuleFinding {
                        rule: RULE_ID.into(), file: relative_slash_path(self.root, self.path),
                        line: crate::codebase::ts_source::byte_offset_to_line(self.source, property.span.start as usize) as usize,
                        message: format!("Vitest declaration {key}: literal {value} is forbidden by {}; use an allowed literal or a computed expression to keep configuration policy explicit",
                            if forbidden { "declared.forbiddenValues" } else { "declared.forbidLiteralKinds" }),
                        import: None, target: None,
                    });
                }
            }
        }
        walk::walk_object_property(self, property);
    }
}

fn equal(left: &serde_json::Value, right: &serde_json::Value) -> bool {
    left == right || (left.is_number() && right.is_number() && left.as_f64() == right.as_f64())
}

fn literal(expression: &Expression<'_>) -> Option<serde_json::Value> {
    match unwrap_ts_wrappers(expression) {
        Expression::BooleanLiteral(value) => Some(value.value.into()),
        Expression::NullLiteral(_) => Some(serde_json::Value::Null),
        Expression::StringLiteral(value) => Some(crate::ast::js_literal(value).as_ref().into()),
        Expression::NumericLiteral(value) => {
            serde_json::Number::from_f64(value.value).map(Into::into)
        }
        Expression::UnaryExpression(unary) => {
            let Expression::NumericLiteral(value) = unwrap_ts_wrappers(&unary.argument) else {
                return None;
            };
            let number = match unary.operator {
                oxc_ast::ast::UnaryOperator::UnaryNegation => -value.value,
                oxc_ast::ast::UnaryOperator::UnaryPlus => value.value,
                _ => return None,
            };
            serde_json::Number::from_f64(number).map(Into::into)
        }
        _ => None,
    }
}
