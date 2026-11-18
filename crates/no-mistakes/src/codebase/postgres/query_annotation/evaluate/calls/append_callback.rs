use super::super::{Environment, Evaluator, Function, Value};
use crate::codebase::postgres::query_annotation::{Expr, Step};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn append_callback_body<'a>(
        &self,
        function: &'a Function,
        inputs: (&[Expr], &[Value]),
        caller: (&Path, Environment),
        owner: (&Path, Environment),
    ) -> Option<&'a [Step]> {
        let proof = function.append_callback.as_ref()?;
        let (path, env) = caller;
        if self.files[owner.0].facts.precise_records
            || !proof
                .tags
                .iter()
                .all(|tag| self.tag_trusted(tag, owner.0, owner.1))
            || !matches!(inputs.0.get(proof.payload), Some(Expr::Tagged(tag, _, _)) if self.tag_trusted(tag, path, env))
            || !matches!(inputs.1.get(proof.payload), Some(Value::Prefix(_, _, Some(id))) if !self.invalidated_builders.contains(id))
        {
            return None;
        }
        Some(&proof.body)
    }
}
