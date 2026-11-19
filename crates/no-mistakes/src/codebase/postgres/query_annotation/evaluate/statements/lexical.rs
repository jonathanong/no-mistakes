use super::super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::{Expr, Step};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    // Lexical facts already answer a direct executor call with a literal leading
    // annotation, and a known helper function only forwards through its own body.
    pub(super) fn lexical_destructure(
        &mut self,
        (step, steps): (&Step, &[Step]),
        path: &Path,
        context: (Environment, u8, bool),
    ) -> bool {
        use crate::codebase::postgres::query_annotation::statements::destructured::{
            literal_annotated_call, RESULT,
        };
        let Step::Bind(name, expr) = step else {
            return false;
        };
        let call = match expr {
            Expr::Await(inner) => inner.as_ref(),
            other => other,
        };
        let Expr::Call {
            callee, start, end, ..
        } = call
        else {
            return false;
        };
        if name != RESULT {
            return false;
        }
        let Expr::Name(callee) = &**callee else {
            return false;
        };
        if self.files[path].executors.contains(&(*start, *end)) {
            let local = steps.iter().any(
                |step| matches!(step, Step::Bind(name, _) | Step::Hoisted(name, _) if name == callee),
            );
            return !local && literal_annotated_call(expr);
        }
        let (_, depth, generic) = context;
        let file = &self.files[path];
        let declared = file.facts.globals.contains_key(callee.as_str())
            || file.module.imports.contains_key(callee.as_str());
        declared && matches!(self.name(path, callee, depth, generic), Value::Function(..))
    }
}
