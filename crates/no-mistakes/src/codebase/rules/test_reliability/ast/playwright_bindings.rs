use super::bindings::Bindings;
use crate::codebase::rules::test_reliability::playwright_provenance::{
    FixtureInitializer, ImportRef, WrapperSummary,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::Expression;
use oxc_semantic::SymbolId;

const MAX_VALUE_DEPTH: usize = 8;

pub(super) enum ValueTrace {
    Resolved(FixtureInitializer),
    Unresolved(&'static str, Vec<ImportRef>),
    None,
}

impl ValueTrace {
    pub(super) fn unresolved(reason: &'static str, candidates: Vec<ImportRef>) -> Self {
        if candidates.is_empty() {
            Self::None
        } else {
            Self::Unresolved(reason, candidates)
        }
    }
}

impl Bindings<'_, '_> {
    pub(super) fn playwright_value_origin(&self, value: &Expression<'_>) -> ValueTrace {
        self.playwright_value_origin_depth(value, 0)
    }

    pub(super) fn playwright_value_origin_depth(
        &self,
        value: &Expression<'_>,
        depth: usize,
    ) -> ValueTrace {
        if depth >= MAX_VALUE_DEPTH {
            return self
                .unresolved_if_provenance(value, "fixture chain exceeded static trace depth");
        }
        let mut derived_at = None;
        let mut wrappers = Vec::new();
        let mut value = unwrap_ts_wrappers(value);
        loop {
            if let Some(member) = value.as_member_expression() {
                if let Some(property) = member.static_property_name().and_then(|n| n.as_str()) {
                    if let Some(import) = self.namespace_member_origin(member.object(), property) {
                        return ValueTrace::Resolved(FixtureInitializer {
                            import,
                            wrappers,
                            derived_at,
                        });
                    }
                }
                return self.unresolved_if_provenance(
                    value,
                    "member access in fixture chain could not be traced statically",
                );
            }
            match value {
                Expression::Identifier(identifier) => {
                    let Some(symbol) = self.symbol(identifier) else {
                        return ValueTrace::None;
                    };
                    if let Some(import) = self.import_origin(symbol, None) {
                        return ValueTrace::Resolved(FixtureInitializer {
                            import,
                            wrappers,
                            derived_at,
                        });
                    }
                    if let Some(init) = self.constant_initializer(identifier) {
                        value = unwrap_ts_wrappers(init);
                        continue;
                    }
                    return self.non_const_or_destructured_trace(symbol);
                }
                Expression::CallExpression(call) if !call.optional => {
                    let callee = unwrap_ts_wrappers(&call.callee);
                    if let Some(member) = callee.as_member_expression() {
                        if member.static_property_name().and_then(|n| n.as_str()) == Some("extend")
                        {
                            derived_at.get_or_insert(call.span.start);
                            value = unwrap_ts_wrappers(member.object());
                            continue;
                        }
                    }
                    let mut visited = FxHashSet::default();
                    if let Some(summary) = self.local_callee_summary(callee, depth, &mut visited) {
                        if let Some(arg) = call
                            .arguments
                            .get(summary.param_index)
                            .and_then(|arg| arg.as_expression())
                        {
                            super::playwright_wrappers::extend_unique(
                                &mut wrappers,
                                summary.wrappers,
                            );
                            derived_at = derived_at.or(summary.derived_at);
                            value = unwrap_ts_wrappers(arg);
                            continue;
                        }
                    }
                    return self.call_value_origin(call, callee, depth, wrappers, derived_at);
                }
                _ => {
                    return self.unresolved_if_provenance(
                        value,
                        "expression in fixture chain could not be traced statically",
                    );
                }
            }
        }
    }

    pub(super) fn local_callee_summary(
        &self,
        callee: &Expression<'_>,
        depth: usize,
        visited: &mut FxHashSet<SymbolId>,
    ) -> Option<WrapperSummary> {
        let Expression::Identifier(identifier) = callee else {
            return None;
        };
        let symbol = self.symbol(identifier)?;
        if self.import_origin(symbol, None).is_some() {
            return None;
        }
        self.wrapper_summary(symbol, depth, visited)
    }

    pub(super) fn callee_import_origin(&self, callee: &Expression<'_>) -> Option<ImportRef> {
        match callee {
            Expression::Identifier(id) => self.import_origin(self.symbol(id)?, None),
            expr => {
                let member = expr.as_member_expression()?;
                let prop = member.static_property_name()?.as_str()?;
                let Expression::Identifier(obj) = unwrap_ts_wrappers(member.object()) else {
                    return None;
                };
                self.import_origin(self.symbol(obj)?, Some(prop))
            }
        }
    }
}
