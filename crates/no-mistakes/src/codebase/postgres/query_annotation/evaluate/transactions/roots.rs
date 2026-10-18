use super::{Cell, Checkpoint};
use crate::codebase::postgres::query_annotation::evaluate::{
    temporaries::TransientRoot, Evaluator, Value,
};
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn collect_transaction_roots(
        &self,
        checkpoint: Checkpoint,
    ) -> Vec<TransientRoot> {
        let mut roots = Vec::new();
        for cell in self.transactions.frames[checkpoint.0].old.keys() {
            self.append_cell_roots(cell, &mut roots);
        }
        roots
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn append_cell_roots(
        &self,
        cell: &Cell,
        roots: &mut Vec<TransientRoot>,
    ) {
        fn collect(value: &Value, roots: &mut Vec<TransientRoot>) {
            match value {
                Value::Function(_, _, env) => roots.push(TransientRoot::Environment(*env)),
                Value::Array(id) | Value::Arguments(id) => roots.push(TransientRoot::Argument(*id)),
                Value::Prefix(_, _, Some(id)) => roots.push(TransientRoot::Builder(*id)),
                Value::Promise(value) | Value::Evaluated(value, _) => collect(value, roots),
                Value::Aggregate(values)
                | Value::References(values)
                | Value::Object(values)
                | Value::Record(values, _)
                | Value::Joined(values)
                | Value::Possible(values) => {
                    for value in values {
                        collect(value, roots);
                    }
                }
                _ => {}
            }
        }
        if let Cell::Module(path) = cell {
            if let Some(env) = self.modules.get(path) {
                roots.push(TransientRoot::Environment(*env));
            }
        }
        if matches!(
            cell,
            Cell::Binding(..) | Cell::Dense(..) | Cell::Extra(..) | Cell::Builder(..)
        ) {
            if let super::Saved::Value(Some(value)) = self.read_cell(cell) {
                collect(&value, roots);
            }
        }
    }
}
