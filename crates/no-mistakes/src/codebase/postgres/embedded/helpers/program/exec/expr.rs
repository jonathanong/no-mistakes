use super::super::{Entry, Expr};
use super::ops::{equals, member, truthiness};
use super::val::{Obj, Path, Text, Val};
use super::{Machine, MAX_EMBEDDED_SQL_VARIANTS};

type Forks = Vec<(Path, Val)>;

impl Machine<'_> {
    /// Every value `expr` can take, each with the path that produced it.
    pub(super) fn eval(&mut self, expr: &Expr, path: Path) -> Option<Forks> {
        let single = |path: Path, value: Val| Some(vec![(path, value)]);
        let forks = match expr {
            Expr::Lit(text) => {
                let value = Val::Str(Text {
                    value: text.sql.clone(),
                    prov: Some(self.stamped(text)),
                });
                return single(path, value);
            }
            Expr::Bool(value) => return single(path, Val::Bool(*value)),
            Expr::Null => return single(path, Val::Null),
            Expr::Undefined => return single(path, Val::Undef),
            Expr::Num => return single(path, Val::Unknown),
            Expr::Ident(name) => {
                let value = path.lookup(name).cloned().unwrap_or(Val::Free);
                return single(path, value);
            }
            Expr::Template(pieces, holes) => self.eval_template(pieces, holes, path)?,
            Expr::Sql(pieces, holes) => self.eval_sql(pieces, holes, path)?,
            Expr::Append(target, arg, mark) => self.eval_append(target, arg, *mark, path)?,
            Expr::Call(name, args) => self.call(name, args, path, false)?,
            Expr::Member(object, property) => {
                let mut out = Vec::new();
                for (path, value) in self.eval(object, path)? {
                    out.push((path, member(&value, property)?));
                }
                out
            }
            Expr::Object(entries) => self.eval_object(entries, path)?,
            Expr::Array(elements) => {
                let exprs: Vec<&Expr> = elements.iter().collect();
                self.eval_all(&exprs, path)?
                    .into_iter()
                    .map(|(path, values)| (path, Val::Arr(values)))
                    .collect()
            }
            Expr::Cond(test, then, otherwise) => {
                let mut out = Vec::new();
                for (path, truth) in self.test(test, path)? {
                    out.extend(self.eval(if truth { then } else { otherwise }, path)?);
                }
                out
            }
            Expr::And(left, right) | Expr::Or(left, right) => {
                let short = matches!(expr, Expr::Or(..));
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
                out
            }
            Expr::Nullish(left, right) => {
                let mut out = Vec::new();
                for (path, value) in self.eval(left, path)? {
                    match value {
                        Val::Null | Val::Undef => out.extend(self.eval(right, path)?),
                        Val::Unknown | Val::Free => {
                            out.push((path.clone(), value));
                            out.extend(self.eval(right, path)?);
                        }
                        other => out.push((path, other)),
                    }
                }
                out
            }
            Expr::Not(inner) => self
                .eval(inner, path)?
                .into_iter()
                .map(|(path, value)| {
                    (
                        path,
                        truthiness(&value).map_or(Val::Unknown, |t| Val::Bool(!t)),
                    )
                })
                .collect(),
            Expr::Eq(left, right, strict, negate) => {
                let mut out = Vec::new();
                for (path, values) in self.eval_all(&[&**left, &**right], path)? {
                    let known = equals(&values[0], &values[1], *strict);
                    out.push((
                        path,
                        known.map_or(Val::Unknown, |eq| Val::Bool(eq != *negate)),
                    ));
                }
                out
            }
            Expr::Add(left, right) => {
                let mut out = Vec::new();
                for (path, values) in self.eval_all(&[&**left, &**right], path)? {
                    out.push((path, concat(&values[0], &values[1])?));
                }
                out
            }
            Expr::Opaque => return None,
        };
        (forks.len() <= MAX_EMBEDDED_SQL_VARIANTS).then_some(forks)
    }

    /// Evaluates `exprs` in order; every combination of their forks.
    pub(super) fn eval_all(
        &mut self,
        exprs: &[&Expr],
        path: Path,
    ) -> Option<Vec<(Path, Vec<Val>)>> {
        let mut states = vec![(path, Vec::with_capacity(exprs.len()))];
        for expr in exprs {
            let mut next = Vec::new();
            for (path, done) in states {
                for (path, value) in self.eval(expr, path)? {
                    let mut done = done.clone();
                    done.push(value);
                    next.push((path, done));
                }
            }
            if next.len() > MAX_EMBEDDED_SQL_VARIANTS {
                return None;
            }
            states = next;
        }
        Some(states)
    }

    /// A statement's expression: an unresolvable call that touches no builder
    /// is ignorable, anything else is evaluated for its effects.
    pub(super) fn eval_statement(&mut self, expr: &Expr, path: Path) -> Option<Forks> {
        match expr {
            Expr::Call(name, args) => self.call(name, args, path, true),
            _ => self.eval(expr, path),
        }
    }

    fn eval_object(&mut self, entries: &[Entry], path: Path) -> Option<Forks> {
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
                        object.props.extend(inner.props);
                        object.open |= inner.open;
                    }
                    (Entry::Spread(_), Val::Null | Val::Undef) => {}
                    (Entry::Spread(_), Val::Unknown) => object.open = true,
                    (Entry::Spread(_), _) => return None,
                }
            }
            out.push((path, Val::Obj(object)));
        }
        Some(out)
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
