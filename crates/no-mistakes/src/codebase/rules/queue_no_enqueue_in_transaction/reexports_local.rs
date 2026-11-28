use super::super::analyze::Context;
use super::{find_import, project_imported, split_export, walk, ResolvedApi};
use crate::codebase::dependencies::extract::ValueExpression;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

struct LocalWalk<'a, F> {
    ctx: &'a Context<'a>,
    target: &'a Path,
    tf: &'a TsFileFacts,
    is_target: &'a F,
    visited: &'a mut FxHashMap<(PathBuf, String), usize>,
}

pub(super) fn classify_local_export<F: Fn(&str, &str) -> bool>(
    ctx: &Context<'_>,
    target: &Path,
    tf: &TsFileFacts,
    local: &str,
    rest: Option<&str>,
    is_target: &F,
    visited: &mut FxHashMap<(PathBuf, String), usize>,
) -> ResolvedApi {
    let mut state = LocalWalk {
        ctx,
        target,
        tf,
        is_target,
        visited,
    };
    state.classify_local(local, rest)
}

impl<F: Fn(&str, &str) -> bool> LocalWalk<'_, F> {
    fn classify_local(&mut self, local: &str, rest: Option<&str>) -> ResolvedApi {
        if let Some(binding) = self
            .tf
            .value_flow
            .bindings
            .iter()
            .find(|b| b.scope == 0 && b.name == local && b.immutable && b.invalidated_at.is_none())
        {
            return self.classify_value_expr(Some(local), &binding.value, rest);
        }
        if is_local_class_binding(self.tf, local)
            || (rest.is_none() && is_local_function(self.tf, local))
        {
            return ResolvedApi::Other;
        }
        ResolvedApi::Unanalyzable
    }

    fn classify_value_expr(
        &mut self,
        local: Option<&str>,
        expr: &ValueExpression,
        rest: Option<&str>,
    ) -> ResolvedApi {
        match expr {
            ValueExpression::Callable(_) if rest.is_none() => ResolvedApi::Other,
            ValueExpression::Object(fields, _) => rest
                .and_then(|member| {
                    let (field, tail) = split_export(member);
                    fields.get(field).map(|field_expr| (field_expr, tail))
                })
                .map_or(ResolvedApi::Unanalyzable, |(field_expr, tail)| {
                    self.classify_value_expr(None, field_expr, tail)
                }),
            ValueExpression::Named { .. } | ValueExpression::Member(..) => {
                self.follow_expr_alias(expr, rest)
            }
            ValueExpression::Unknown
                if local.is_some_and(|name| {
                    is_local_class_binding(self.tf, name)
                        || (rest.is_none() && is_local_function(self.tf, name))
                }) =>
            {
                ResolvedApi::Other
            }
            _ => ResolvedApi::Unanalyzable,
        }
    }

    fn follow_expr_alias(&mut self, expr: &ValueExpression, rest: Option<&str>) -> ResolvedApi {
        let mut parts = Vec::new();
        let mut cur = expr;
        while let ValueExpression::Member(inner, prop) = cur {
            parts.push(prop.as_str());
            cur = inner.as_ref();
        }
        let ValueExpression::Named {
            name,
            scope: None | Some(0),
            ..
        } = cur
        else {
            return ResolvedApi::Unanalyzable;
        };
        parts.reverse();
        if let Some(tail) = rest {
            parts.push(tail);
        }
        let suffix = (!parts.is_empty()).then(|| parts.join("."));
        if let Some(imp) = find_import(self.tf, name) {
            return project_imported(imp, suffix.as_deref()).map_or(
                ResolvedApi::Unanalyzable,
                |next| {
                    walk(
                        self.ctx,
                        self.target,
                        &imp.specifier,
                        &next,
                        self.is_target,
                        self.visited,
                    )
                },
            );
        }
        let visit_key = match suffix.as_deref() {
            Some(tail) => format!("\0{name}.{tail}"),
            None => format!("\0{name}"),
        };
        if self.visited.insert((self.target.to_path_buf(), visit_key), 0).is_some() {
            return ResolvedApi::Unanalyzable;
        }
        self.classify_local(name, suffix.as_deref())
    }
}

fn is_local_function(tf: &TsFileFacts, local: &str) -> bool {
    tf.exported_functions.iter().any(|scope| scope == local)
}

fn is_local_class_binding(tf: &TsFileFacts, local: &str) -> bool {
    tf.class_scopes.iter().any(|scope| scope == local)
        || tf
            .callable_bindings
            .iter()
            .find(|(scope, name, _)| *scope == 0 && name == local)
            .is_some_and(|(_, _, id)| {
                tf.callable_scope_ids
                    .iter()
                    .any(|(cid, scope)| cid == id && tf.class_scopes.contains(scope))
            })
}
