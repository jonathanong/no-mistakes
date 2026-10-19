//! Compound-expression evaluation: one helper per `Expr` kind, so `eval`
//! stays a flat dispatch.

use super::super::{Entry, Expr};
use super::ops::{equals, member, truthiness};
use super::val::{Obj, Path, Text, Val};
use super::Machine;

type Forks = Vec<(Path, Val)>;

impl Machine<'_> {
    pub(super) fn eval_member(
        &mut self,
        object: &Expr,
        property: &str,
        path: Path,
    ) -> Option<Forks> {
        let mut out = Vec::new();
        for (path, value) in self.eval(object, path)? {
            out.push((path, member(&value, property)?));
        }
        Some(out)
    }

    pub(super) fn eval_array(&mut self, elements: &[Expr], path: Path) -> Option<Forks> {
        let exprs: Vec<&Expr> = elements.iter().collect();
        Some(
            self.eval_all(&exprs, path)?
                .into_iter()
                .map(|(path, values)| (path, Val::Arr(values)))
                .collect(),
        )
    }

    pub(super) fn eval_cond(
        &mut self,
        test: &Expr,
        then: &Expr,
        otherwise: &Expr,
        path: Path,
    ) -> Option<Forks> {
        let mut out = Vec::new();
        for (path, truth) in self.test(test, path)? {
            out.extend(self.eval(if truth { then } else { otherwise }, path)?);
        }
        Some(out)
    }

    /// `&&` (`short` false) and `||` (`short` true).
    pub(super) fn eval_logic(
        &mut self,
        left: &Expr,
        right: &Expr,
        short: bool,
        path: Path,
    ) -> Option<Forks> {
        let mut out = Vec::new();
        for (path, value) in self.eval(left, path)? {
            match truthiness(&value) {
                Some(truth) if truth == short => out.push((path, value)),
                Some(_) => out.extend(self.eval(right, path)?),
                None => {
                    // An unknown left side forks, and remembers which
                    // way it went for the next identical test.
                    for (path, truth) in self.settle(left, path, &value) {
                        if truth == short {
                            out.push((path, Val::Unknown));
                        } else {
                            out.extend(self.eval(right, path)?);
                        }
                    }
                }
            }
        }
        Some(out)
    }

    pub(super) fn eval_nullish(&mut self, left: &Expr, right: &Expr, path: Path) -> Option<Forks> {
        let mut out = Vec::new();
        for (path, value) in self.eval(left, path)? {
            match value {
                Val::Null | Val::Undef => out.extend(self.eval(right, path)?),
                // A bound scalar stays a bind. SQL on the right keeps both forks.
                Val::Unknown => {
                    out.extend(nullish_unknown(self.eval(right, path.clone())?, path));
                }
                // Free is not a bound value, so the fallback is not published.
                Val::Free => out.push((path, value)),
                other => out.push((path, other)),
            }
        }
        Some(out)
    }

    pub(super) fn eval_not(&mut self, inner: &Expr, path: Path) -> Option<Forks> {
        Some(
            self.eval(inner, path)?
                .into_iter()
                .map(|(path, value)| {
                    (
                        path,
                        truthiness(&value).map_or(Val::Unknown, |t| Val::Bool(!t)),
                    )
                })
                .collect(),
        )
    }

    pub(super) fn eval_eq(
        &mut self,
        left: &Expr,
        right: &Expr,
        strict: bool,
        negate: bool,
        path: Path,
    ) -> Option<Forks> {
        let mut out = Vec::new();
        for (path, values) in self.eval_all(&[left, right], path)? {
            let known = equals(&values[0], &values[1], strict);
            out.push((
                path,
                known.map_or(Val::Unknown, |eq| Val::Bool(eq != negate)),
            ));
        }
        Some(out)
    }

    pub(super) fn eval_add(&mut self, left: &Expr, right: &Expr, path: Path) -> Option<Forks> {
        let mut out = Vec::new();
        for (path, values) in self.eval_all(&[left, right], path)? {
            out.push((path, concat(&values[0], &values[1])?));
        }
        Some(out)
    }

    pub(super) fn eval_object(&mut self, entries: &[Entry], path: Path) -> Option<Forks> {
        let exprs: Vec<&Expr> = entries
            .iter()
            .map(|entry| match entry {
                Entry::Prop(_, value) | Entry::Spread(value) => value,
            })
            .collect();
        let mut out = Vec::new();
        for (path, values) in self.eval_all(&exprs, path)? {
            let mut object = Obj {
                props: Vec::new(),
                open: false,
            };
            for (entry, value) in entries.iter().zip(values) {
                match (entry, value) {
                    (Entry::Prop(key, _), value) => object.props.push((key.clone(), value)),
                    (Entry::Spread(_), Val::Obj(inner)) => {
                        if inner.open {
                            unknown_props(&mut object);
                        }
                        object.props.extend(inner.props);
                        object.open |= inner.open;
                    }
                    (Entry::Spread(_), Val::Null | Val::Undef) => {}
                    (Entry::Spread(_), Val::Unknown) => unknown_props(&mut object),
                    (Entry::Spread(_), _) => return None,
                }
            }
            out.push((path, Val::Obj(object)));
        }
        Some(out)
    }
}

/// Unknown `??` right. SQL keeps the left fork and the right forks. Any other
/// right, including a module const the body does not bind, collapses to one bind.
fn nullish_unknown(forks: Forks, path: Path) -> Forks {
    if forks.iter().any(|(_, value)| matches!(value, Val::Sql(_))) {
        let mut kept = Vec::with_capacity(forks.len() + 1);
        kept.push((path, Val::Unknown));
        kept.extend(forks);
        return kept;
    }
    forks
        .into_iter()
        .map(|(path, _)| (path, Val::Unknown))
        .collect()
}

/// A spread of unknown keys may override any earlier prop, so none of them
/// stays known; props written after the spread are unaffected.
fn unknown_props(object: &mut Obj) {
    object.open = true;
    for (_, value) in &mut object.props {
        *value = Val::Unknown;
    }
}

/// `left + right`: concatenation of known strings, or an unknown scalar.
fn concat(left: &Val, right: &Val) -> Option<Val> {
    Some(match (left, right) {
        (Val::Str(a), Val::Str(b)) => Val::Str(Text {
            value: format!("{}{}", a.value, b.value),
            prov: None,
        }),
        (Val::Sql(_) | Val::Free | Val::Obj(_), _) | (_, Val::Sql(_) | Val::Free | Val::Obj(_)) => {
            return None
        }
        _ => Val::Unknown,
    })
}
