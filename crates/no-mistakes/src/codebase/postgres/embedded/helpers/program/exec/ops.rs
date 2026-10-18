//! Truthiness, equality, property access and test forking.

use super::super::Expr;
use super::val::{Obj, Path, Val};
use super::Machine;

/// `Some` when the value's truthiness is known.
pub(super) fn truthiness(value: &Val) -> Option<bool> {
    match value {
        Val::Str(text) => Some(!text.value.is_empty()),
        Val::Bool(value) => Some(*value),
        Val::Null | Val::Undef => Some(false),
        Val::Obj(_) | Val::Arr(_) | Val::Sql(_) => Some(true),
        Val::Unknown | Val::Free => None,
    }
}

/// `Some` when `left == right` is known. Only primitives are compared.
pub(super) fn equals(left: &Val, right: &Val, strict: bool) -> Option<bool> {
    let nullish = |value: &Val| matches!(value, Val::Null | Val::Undef);
    if !strict && (nullish(left) || nullish(right)) {
        return match (left, right) {
            (Val::Unknown | Val::Free, _) | (_, Val::Unknown | Val::Free) => None,
            _ => Some(nullish(left) && nullish(right)),
        };
    }
    match (left, right) {
        (Val::Str(a), Val::Str(b)) => Some(a.value == b.value),
        (Val::Bool(a), Val::Bool(b)) => Some(a == b),
        (Val::Null, Val::Null) | (Val::Undef, Val::Undef) => Some(true),
        (Val::Unknown | Val::Free, _) | (_, Val::Unknown | Val::Free) => None,
        (Val::Obj(_) | Val::Arr(_) | Val::Sql(_), _)
        | (_, Val::Obj(_) | Val::Arr(_) | Val::Sql(_)) => None,
        // Different primitive kinds are never strictly equal.
        _ if strict => Some(false),
        _ => None,
    }
}

/// `value.property`; `None` when the access is not modelled.
pub(super) fn member(value: &Val, property: &str) -> Option<Val> {
    match value {
        Val::Obj(Obj { props, open }) => Some(
            props
                .iter()
                .rev()
                .find(|(key, _)| key == property)
                .map(|(_, value)| value.clone())
                .unwrap_or(if *open { Val::Unknown } else { Val::Undef }),
        ),
        Val::Null | Val::Undef => Some(Val::Undef),
        Val::Unknown => Some(Val::Unknown),
        Val::Free => Some(Val::Free),
        Val::Arr(_) if property == "length" => Some(Val::Unknown),
        Val::Str(_) | Val::Bool(_) | Val::Arr(_) | Val::Sql(_) => None,
    }
}

impl Machine<'_> {
    /// Both outcomes of a test, each with the path that took it. A test on
    /// parameters only is remembered so a repeat agrees with the first.
    pub(super) fn test(&mut self, expr: &Expr, path: Path) -> Option<Vec<(Path, bool)>> {
        match expr {
            Expr::Not(inner) => Some(
                self.test(inner, path)?
                    .into_iter()
                    .map(|(path, truth)| (path, !truth))
                    .collect(),
            ),
            Expr::Eq(left, right, strict, true) => {
                let positive = Expr::Eq(left.clone(), right.clone(), *strict, false);
                Some(
                    self.test(&positive, path)?
                        .into_iter()
                        .map(|(path, truth)| (path, !truth))
                        .collect(),
                )
            }
            Expr::And(left, right) | Expr::Or(left, right) => {
                let short = matches!(expr, Expr::Or(..));
                let mut out = Vec::new();
                for (path, truth) in self.test(left, path)? {
                    if truth == short {
                        out.push((path, short));
                    } else {
                        out.extend(self.test(right, path)?);
                    }
                }
                Some(out)
            }
            _ => {
                let mut out = Vec::new();
                for (path, value) in self.eval(expr, path)? {
                    out.extend(self.settle(expr, path, &value));
                }
                Some(out)
            }
        }
    }

    pub(super) fn settle(&self, expr: &Expr, path: Path, value: &Val) -> Vec<(Path, bool)> {
        if let Some(truth) = truthiness(value) {
            return vec![(path, truth)];
        }
        let key = self.key(expr);
        if let Some(truth) = key.as_deref().and_then(|key| path.decision(key)) {
            return vec![(path, truth)];
        }
        [true, false]
            .into_iter()
            .map(|truth| {
                let mut fork = path.clone();
                fork.decide(key.clone(), truth);
                (fork, truth)
            })
            .collect()
    }

    /// Identity of a test built only from parameters and literals.
    fn key(&self, expr: &Expr) -> Option<String> {
        Some(match expr {
            Expr::Ident(name) if self.params.contains(name) => name.clone(),
            Expr::Member(object, property) => format!("{}.{property}", self.key(object)?),
            Expr::Lit(text) => format!("'{}'", text.sql),
            Expr::Bool(value) => value.to_string(),
            Expr::Null => "null".into(),
            Expr::Undefined => "undefined".into(),
            Expr::Eq(left, right, strict, false) => {
                format!(
                    "({}{}{})",
                    self.key(left)?,
                    if *strict { "===" } else { "==" },
                    self.key(right)?
                )
            }
            _ => return None,
        })
    }
}
