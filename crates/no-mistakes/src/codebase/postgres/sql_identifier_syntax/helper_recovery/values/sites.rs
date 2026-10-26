//! Replay caller locals only for the exact actual sink, never a neighboring call.
use super::super::{CallModel, Recovered};
use super::*;
use crate::{
    codebase::postgres::embedded::helpers::program::{Expr, Stmt},
    fx::FxHashSet,
};

impl Engine<'_> {
    pub fn site(
        &mut self,
        file: &Path,
        call: &CallModel,
        executors: &FxHashSet<usize>,
    ) -> Option<Recovered> {
        let key = Key {
            file: file.to_path_buf(),
            callable: call.caller?,
        };
        let program = self.catalog.function(&key)?;
        let args = program
            .program
            .params
            .iter()
            .map(|param| {
                let direct = param
                    .name
                    .strip_prefix("binding:")
                    .and_then(|id| id.parse().ok())
                    .is_some_and(|id| executors.contains(&id));
                let receiver = program.program.body.iter().any(|statement| {
                    let Stmt::Let(name, Expr::Member(root, member)) = statement else {
                        return false;
                    };
                    let Expr::Ident(root) = root.as_ref() else {
                        return false;
                    };
                    root == &param.name
                        && member == "query"
                        && name
                            .strip_prefix("binding:")
                            .and_then(|id| id.parse().ok())
                            .is_some_and(|id| executors.contains(&id))
                });
                if direct || receiver {
                    Value::Executor
                } else {
                    Value::Scalar
                }
            })
            .collect::<Vec<_>>();
        self.effects = true;
        self.root_file = Some(file.to_path_buf());
        if self.run(&key, &args).is_none() {
            self.failures += 1;
        }
        let invalid_bind = self
            .executed
            .iter()
            .filter(|event| event.file == file && event.start == call.span.0)
            .any(|event| !event.sql.valid_binds());
        let (alternatives, execution_sites): (Vec<_>, Vec<_>) = self
            .executed
            .iter()
            .filter(|event| event.file == file && event.start == call.span.0)
            .map(|event| (event.sql.raw.clone(), Some(event.site())))
            .unzip();
        (!alternatives.is_empty()).then(|| Recovered {
            alternatives,
            execution_sites,
            residual: self.residual() || invalid_bind,
            nonexecuting: false,
        })
    }
}
