use super::super::{Expr, Mark, Stmt, SwitchArm};
use super::val::{Path, Val};
use super::{Machine, Outcome, Ret, MAX_EMBEDDED_SQL_VARIANTS};

impl Machine<'_> {
    /// Runs `stmts` on every live path. A returned path skips the rest.
    pub(super) fn block(&mut self, stmts: &[Stmt], paths: Vec<Path>) -> Option<Vec<Path>> {
        let mut paths = paths;
        for statement in stmts {
            let mut next = Vec::with_capacity(paths.len());
            for path in paths {
                if path.returned.is_some() {
                    next.push(path);
                } else {
                    next.extend(self.statement(statement, path)?);
                }
            }
            if next.len() > MAX_EMBEDDED_SQL_VARIANTS {
                return None;
            }
            paths = next;
        }
        Some(paths)
    }

    fn statement(&mut self, statement: &Stmt, path: Path) -> Option<Vec<Path>> {
        match statement {
            Stmt::Let(name, init) => Some(
                self.eval(init, path)?
                    .into_iter()
                    .map(|(mut path, value)| {
                        path.bind(name, value);
                        path
                    })
                    .collect(),
            ),
            Stmt::Expr(expr) => Some(
                self.eval_statement(expr, path)?
                    .into_iter()
                    .map(|(path, _)| path)
                    .collect(),
            ),
            Stmt::If(test, then, otherwise) => {
                let mut out = Vec::new();
                for (mut path, truth) in self.test(test, path)? {
                    path.push_scope();
                    let taken = if truth { then } else { otherwise };
                    for mut done in self.block(taken, vec![path])? {
                        done.pop_scope();
                        out.push(done);
                    }
                }
                Some(out)
            }
            Stmt::Return(None, _) => Some(vec![returned(path, Val::Undef)]),
            Stmt::Return(Some(expr), mark) => Some(
                self.eval(expr, path)?
                    .into_iter()
                    .map(|(path, value)| {
                        let value = self.with_provenance(value, *mark);
                        returned(path, value)
                    })
                    .collect(),
            ),
            Stmt::Throw => Some(Vec::new()),
            Stmt::ForOf(name, iterable, body) => self.for_of(name, iterable, body, path),
            Stmt::Switch(discriminant, arms) => self.switch(discriminant, arms, path),
            Stmt::Opaque => None,
        }
    }

    /// Unknown selectors keep every arm. Past the cap the call abstains.
    #[rustfmt::skip]
    fn switch(&mut self, discriminant: &Expr, arms: &[SwitchArm], path: Path) -> Option<Vec<Path>> {
        let mut out = Vec::new();
        for (path, selector) in self.eval(discriminant, path)? {
            match selector {
                Val::Str(text) => {
                    if let Some(arm) = arms.iter().find(|arm| arm.cases.iter().any(|case| case == &text.value)) {
                        out.extend(self.switch_arm(arm, path)?);
                    }
                }
                Val::Unknown | Val::Free => {
                    for arm in arms { out.extend(self.switch_arm(arm, path.clone())?); }
                }
                _ => {}
            }
            if out.len() > MAX_EMBEDDED_SQL_VARIANTS { return None; }
        }
        Some(out)
    }

    #[rustfmt::skip]
    fn switch_arm(&mut self, arm: &SwitchArm, path: Path) -> Option<Vec<Path>> {
        Some(self.eval(&arm.value, path)?.into_iter().map(|(path, value)| {
            returned(path, self.with_provenance(value, arm.mark))
        }).collect())
    }

    /// The caller-visible result of a finished path.
    pub(super) fn outcome(&self, path: Path, builders: &[(usize, usize)]) -> Option<Outcome> {
        let suffixes = builders
            .iter()
            .map(|(param, heap)| (*param, path.heap[*heap].text.clone()))
            .collect();
        let ret = match &path.returned {
            Some(Val::Sql(id)) => match path.heap[*id].param {
                Some(param) => Ret::Param(param),
                None => Ret::Sql(path.heap[*id].text.clone()),
            },
            Some(Val::Str(text)) => Ret::Str(text.value.clone(), text.prov.clone()),
            _ => Ret::Void,
        };
        Some(Outcome { ret, suffixes })
    }
}

fn returned(mut path: Path, value: Val) -> Path {
    path.returned = Some(value);
    path
}

impl Machine<'_> {
    /// A string produced from caller data reads from the returning expression.
    fn with_provenance(&self, value: Val, mark: Mark) -> Val {
        match value {
            Val::Str(mut text) if text.prov.is_none() => {
                text.prov = self.plain_text(&text.value, mark);
                Val::Str(text)
            }
            other => other,
        }
    }
}
