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
        context: (u8, bool, bool),
    ) -> Option<Value> {
        let (depth, generic, operands_handled) = context;
        if self.files[path].facts.precise_records {
            if let Expr::Member(_, name) = callee {
                if matches!(name.as_str(), "map" | "join" | "push" | "forEach")
                    && args.iter().any(|arg| matches!(arg, Expr::Spread(_)))
                {
                    self.invalidate_live_builders();
                    self.opaque_callbacks(arguments, depth);
                    return Some(Value::Failure("opaque-spread"));
                }
                if matches!(name.as_str(), "map" | "join" | "push" | "forEach") && !operands_handled
                {
                    self.invalidate_live_builders();
                    self.opaque_callbacks(arguments, depth);
                    let reason = receiver
                        .into_iter()
                        .chain(arguments)
                        .find_map(|value| match value {
                            Value::Failure(reason) => Some(*reason),
                            _ => None,
                        })
                        .unwrap_or("opaque-collection-operand-effect");
                    return Some(Value::Failure(reason));
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
                self.create_argument_object(id, values.to_vec());
                Value::Arguments(id)
            }
        }
    }
}
