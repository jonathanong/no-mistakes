use super::super::Expr;
use super::val::{Path, Text, Val};
use super::{Machine, MAX_EMBEDDED_SQL_VARIANTS};

type Forks = Vec<(Path, Val)>;

impl Machine<'_> {
    /// Every value `expr` can take, each with the path that produced it.
    pub(super) fn eval(&mut self, expr: &Expr, path: Path) -> Option<Forks> {
        let forks = match expr {
            Expr::Lit(_)
            | Expr::Bool(_)
            | Expr::Null
            | Expr::Undefined
            | Expr::Num
            | Expr::Ident(_) => return self.eval_leaf(expr, path),
            Expr::Template(pieces, holes) => self.eval_template(pieces, holes, path),
            Expr::Sql(pieces, holes) => self.eval_sql(pieces, holes, path),
            Expr::Append(target, arg, mark) => self.eval_append(target, arg, *mark, path),
            Expr::Call(name, args) => self.call(name, args, path, false),
            Expr::Member(object, property) => self.eval_member(object, property, path),
            Expr::Object(entries) => self.eval_object(entries, path),
            Expr::Array(elements) => self.eval_array(elements, path),
            Expr::Cond(test, then, otherwise) => self.eval_cond(test, then, otherwise, path),
            Expr::And(left, right) => self.eval_logic(left, right, false, path),
            Expr::Or(left, right) => self.eval_logic(left, right, true, path),
            Expr::Nullish(left, right) => self.eval_nullish(left, right, path),
            Expr::Not(inner) => self.eval_not(inner, path),
            Expr::Eq(left, right, strict, negate) => {
                self.eval_eq(left, right, *strict, *negate, path)
            }
            Expr::Add(left, right) => self.eval_add(left, right, path),
            Expr::Opaque => return None,
        }?;
        (forks.len() <= MAX_EMBEDDED_SQL_VARIANTS).then_some(forks)
    }

    /// Literals and identifiers: exactly one fork, never capped.
    fn eval_leaf(&mut self, expr: &Expr, path: Path) -> Option<Forks> {
        let single = |path: Path, value: Val| Some(vec![(path, value)]);
        match expr {
            Expr::Lit(text) => {
                let value = Val::Str(Text {
                    value: text.sql.clone(),
                    prov: Some(self.stamped(text)),
                });
                single(path, value)
            }
            Expr::Bool(value) => single(path, Val::Bool(*value)),
            Expr::Null => single(path, Val::Null),
            Expr::Undefined => single(path, Val::Undef),
            Expr::Num => single(path, Val::Unknown),
            Expr::Ident(name) => {
                let value = path.lookup(name).cloned().unwrap_or(Val::Free);
                single(path, value)
            }
            _ => None,
        }
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
}
