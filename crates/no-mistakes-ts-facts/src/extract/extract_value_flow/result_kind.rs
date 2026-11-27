use super::*;

impl ImportCollector {
    pub(in super::super) fn record_callable_result_kind(
        &mut self,
        is_async: bool,
        generator: bool,
    ) {
        if self.collect_value_flow {
            if let Some(owner) = self.current_function_id() {
                if is_async {
                    self.value_flow.async_callables.insert(owner);
                }
                if generator {
                    self.value_flow.generator_callables.insert(owner);
                }
            }
        }
    }
}
