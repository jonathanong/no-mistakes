//! Calls from one helper into another.

use super::super::Expr;
use super::val::{empty_sql, Obj, Path, Text, Val};
use super::{Callee, Machine, Outcome, Ret};
use crate::codebase::postgres::embedded::helpers::split::compose;
use crate::codebase::postgres::embedded::helpers::{LiteralArg, SiteArg};

/// What a callee is told about an argument value. `None` for a value whose
/// nature is unknown (a name the helper does not bind).
fn site_arg(value: &Val) -> Option<SiteArg> {
    Some(match value {
        Val::Str(text) => SiteArg::Literal(LiteralArg::bare(text.value.clone())),
        Val::Bool(value) => SiteArg::Bool(*value),
        Val::Null => SiteArg::Null,
        Val::Undef => SiteArg::Undefined,
        Val::Unknown => SiteArg::Scalar,
        Val::Free => return None,
        Val::Sql(_) => SiteArg::Builder,
        Val::Arr(items) => SiteArg::Array(items.iter().map(nested_arg).collect::<Option<_>>()?),
        Val::Obj(Obj { open: true, .. }) => SiteArg::Scalar,
        Val::Obj(Obj { props, .. }) => SiteArg::Object(
            props
                .iter()
                .map(|(key, value)| Some((key.clone(), nested_arg(value)?)))
                .collect::<Option<_>>()?,
        ),
    })
}

/// A value inside an object or array. A builder hidden in there could be
/// mutated by the callee without the caller seeing it, so none is accepted.
fn nested_arg(value: &Val) -> Option<SiteArg> {
    match value {
        Val::Sql(_) => None,
        other => site_arg(other),
    }
}

impl Machine<'_> {
    /// Calls helper `name`. As a statement, a callee with no program that is
    /// handed no builder is ignored: it cannot have changed any SQL.
    pub(super) fn call(
        &mut self,
        name: &str,
        args: &[Expr],
        path: Path,
        statement: bool,
    ) -> Option<Vec<(Path, Val)>> {
        // A parameter or local of that name is a value, not the module's helper.
        if path.lookup(name).is_some() {
            return None;
        }
        let exprs: Vec<&Expr> = args.iter().collect();
        let mut out = Vec::new();
        for (path, values) in self.eval_all(&exprs, path)? {
            if repeats_builder(&values) {
                return None;
            }
            let site = values.iter().map(site_arg).collect::<Option<Vec<_>>>()?;
            match self.callees.call(name, &site) {
                Callee::Failed => return None,
                Callee::Unresolved => {
                    if !statement || !site.iter().all(primitive) {
                        return None;
                    }
                    out.push((path, Val::Unknown));
                }
                Callee::Done(outcomes) => {
                    for outcome in outcomes {
                        out.push(apply(path.clone(), &values, outcome)?);
                    }
                }
            }
        }
        Some(out)
    }
}

/// The same builder at two parameters: the callee's appends would alias.
fn repeats_builder(values: &[Val]) -> bool {
    let ids: Vec<usize> = values
        .iter()
        .filter_map(|value| match value {
            Val::Sql(id) => Some(*id),
            _ => None,
        })
        .collect();
    ids.iter().enumerate().any(|(i, id)| ids[..i].contains(id))
}

/// A value a callee cannot use to reach the caller's SQL state.
fn primitive(arg: &SiteArg) -> bool {
    matches!(
        arg,
        SiteArg::Literal(_)
            | SiteArg::Bool(_)
            | SiteArg::Null
            | SiteArg::Undefined
            | SiteArg::Scalar
    )
}

fn apply(mut path: Path, values: &[Val], outcome: Outcome) -> Option<(Path, Val)> {
    for (index, suffix) in outcome.suffixes {
        let Some(Val::Sql(id)) = values.get(index) else {
            continue;
        };
        let base = std::mem::replace(&mut path.heap[*id].text, empty_sql());
        path.heap[*id].text = compose([base, suffix]).unwrap_or_else(empty_sql);
    }
    let value = match outcome.ret {
        Ret::Void => Val::Undef,
        Ret::Str(value, prov) => Val::Str(Text { value, prov }),
        Ret::Sql(text) => path.new_builder(text, None),
        Ret::Param(index) => values.get(index)?.clone(),
    };
    Some((path, value))
}
