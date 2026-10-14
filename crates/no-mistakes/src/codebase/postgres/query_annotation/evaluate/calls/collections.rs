use super::super::{Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn collection_projection(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        receiver: Option<&Value>,
        arguments: &[Value],
        path: &Path,
        context: (u8, bool),
    ) -> Option<Value> {
        let (depth, generic) = context;
        if self.files[path].facts.precise_records {
            if let Expr::Member(_, name) = callee {
                if matches!(name.as_str(), "map" | "join" | "push" | "forEach")
                    && args.iter().any(|arg| matches!(arg, Expr::Spread(_)))
                {
                    self.invalidate_live_builders();
                    self.opaque_callbacks(arguments, depth);
                    return Some(Value::Failure("opaque-spread"));
                }
                if let Some(value) =
                    self.collection_call(receiver, name, arguments, path, (depth, generic))
                {
                    return Some(value);
                }
            }
        }
        None
    }
    pub(super) fn arguments_object(&mut self, values: &[Value], spread: bool) -> Value {
        if spread {
            Value::Unknown
        } else {
            let id = self.next_builder;
            self.next_builder += 1;
            {
                self.argument_objects.insert(id, values.to_vec());
                Value::Arguments(id)
            }
        }
    }
}
