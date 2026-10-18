//! Production checkpoints carry no oracle flag or validation work.
use super::super::Evaluator;

#[derive(Default)]
pub(in crate::codebase::postgres::query_annotation::evaluate) struct State;

impl<F> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation) fn with_checkpoint_index_oracle(
        self,
    ) -> Self {
        self
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn assert_checkpoint_indexes(
        &self,
    ) {
        let _ = &self.transactions.index_oracle;
    }
}
