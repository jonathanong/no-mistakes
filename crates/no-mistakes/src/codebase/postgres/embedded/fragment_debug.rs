use super::EmbeddedSqlFileFacts;

// Fragment alternatives are internal provenance, so the legacy public debug
// surface remains byte-identical even when precise fragment origins exist.
impl std::fmt::Debug for EmbeddedSqlFileFacts {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EmbeddedSqlFileFacts")
            .field("path", &self.path)
            .field("executor_bindings", &self.executor_bindings)
            .field("calls", &self.calls)
            .field("call_spans", &self.call_spans)
            .field("fragments", &self.fragments)
            .field("matched_factory_names", &self.matched_factory_names)
            .field("matched_type_names", &self.matched_type_names)
            .field("pending_relative", &self.pending_relative)
            .finish()
    }
}
