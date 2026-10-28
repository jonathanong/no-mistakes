use crate::codebase::{
    dependencies::extract::CallableId,
    postgres::embedded::walk::resolve::functions::provenance::RecoveredSql,
};
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};

pub(super) struct Collector<'a> {
    pub source: &'a str,
    pub slots: FxHashMap<(u32, u32), usize>,
    pub runners: FxHashSet<usize>,
    pub sites: FxHashMap<CallableId, Vec<RecoveredSql>>,
    pub runner_uses: FxHashSet<(u32, u32)>,
    pub unsafe_file: bool,
}

impl Collector<'_> {
    fn runner(&self, expression: &Expression<'_>) -> bool {
        let Expression::Identifier(identifier) = expression else {
            return false;
        };
        self.slots
            .get(&(identifier.span.start, identifier.span.end))
            .is_some_and(|binding| self.runners.contains(binding))
    }

    fn collect_outer_call(&mut self, outer: &CallExpression<'_>) {
        let Expression::CallExpression(each) = &outer.callee else {
            return;
        };
        let Expression::StaticMemberExpression(member) = &each.callee else {
            return;
        };
        if member.property.name != "each"
            || !self.runner(&member.object)
            || each.optional
            || each.arguments.len() != 1
            || outer.optional
            || outer.arguments.len() != 2
        {
            return;
        }
        let Some(rows) = each.arguments[0]
            .as_expression()
            .and_then(|expr| self.rows(expr))
        else {
            return;
        };
        let Some(callback) = outer.arguments[1]
            .as_expression()
            .and_then(|arg| match arg {
                Expression::ArrowFunctionExpression(callback) => Some(callback),
                _ => None,
            })
        else {
            return;
        };
        if !matches!(callback.body, ArrowFunctionBody::FunctionBody(_))
            || callback.params.items.len() != 1
            || callback.params.rest.is_some()
            || callback.params.items[0].initializer.is_some()
            || !matches!(
                callback.params.items[0].pattern,
                BindingPattern::BindingIdentifier(_)
            )
        {
            return;
        }
        self.sites.insert(CallableId(callback.span.start), rows);
    }

    fn rows(&self, expression: &Expression<'_>) -> Option<Vec<RecoveredSql>> {
        let expression = unwrap(expression);
        let Expression::ArrayExpression(array) = expression else {
            return None;
        };
        if array.elements.is_empty() || array.elements.len() > 16 {
            return None;
        }
        array
            .elements
            .iter()
            .map(|element| {
                let Expression::StringLiteral(literal) = element.as_expression()? else {
                    return None;
                };
                let raw = self
                    .source
                    .get(literal.span.start as usize + 1..literal.span.end as usize - 1)?;
                let sql = crate::ast::js_literal(literal).into_owned();
                // Escape decoding would need a separate exact source map.
                if raw != sql || sql.contains(['\n', '\r']) {
                    return None;
                }
                let line = crate::codebase::ts_source::byte_offset_to_line(
                    self.source,
                    literal.span.start as usize,
                );
                Some(RecoveredSql {
                    sql: sql.to_owned(),
                    line,
                    positions: vec![
                        crate::codebase::postgres::embedded::EmbeddedSqlSourcePosition {
                            sql_line: 1,
                            sql_column: 1,
                            source_line: line,
                            source_path: None,
                        },
                    ],
                    origins: (0..sql.len())
                        .map(|offset| literal.span.start + 1 + offset as u32)
                        .collect(),
                })
            })
            .collect()
    }
}

fn unwrap<'e, 'a>(mut expression: &'e Expression<'a>) -> &'e Expression<'a> {
    loop {
        expression = match expression {
            Expression::TSAsExpression(ts) => &ts.expression,
            Expression::TSSatisfiesExpression(ts) => &ts.expression,
            Expression::TSNonNullExpression(ts) => &ts.expression,
            Expression::ParenthesizedExpression(parenthesized) => &parenthesized.expression,
            _ => return expression,
        };
    }
}

impl<'a> Visit<'a> for Collector<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        match &call.callee {
            Expression::Identifier(identifier) if self.runner(&call.callee) => {
                self.runner_uses
                    .insert((identifier.span.start, identifier.span.end));
            }
            Expression::StaticMemberExpression(member)
                if member.property.name == "each" && self.runner(&member.object) =>
            {
                if let Expression::Identifier(identifier) = &member.object {
                    self.runner_uses
                        .insert((identifier.span.start, identifier.span.end));
                }
            }
            _ => {}
        }
        if matches!(&call.callee, Expression::Identifier(identifier) if identifier.name == "eval") {
            self.unsafe_file = true;
        }
        self.collect_outer_call(call);
        walk::walk_call_expression(self, call);
    }

    fn visit_with_statement(&mut self, _: &WithStatement<'a>) {
        self.unsafe_file = true;
    }
}
