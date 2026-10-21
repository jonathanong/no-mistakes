#[cfg(not(test))]
macro_rules! record_expression_call_fact {
    () => {};
}

#[cfg(not(test))]
macro_rules! record_unknown_call_fact {
    () => {};
}

#[cfg(test)]
#[macro_use]
#[path = "extract_graph_call_recording_probe.rs"]
pub(crate) mod graph_call_recording_probe;

impl ImportCollector {
    fn record_expression_call_fact(&mut self, call: FunctionCall) {
        record_expression_call_fact!();
        self.function_calls.push(call);
    }

    fn record_unknown_call(&mut self, line: u32, offset: u32, invocation: InvocationKind) {
        record_expression_call_fact!();
        record_unknown_call_fact!();
        let caller = self.current_function();
        if caller.is_none() {
            self.has_unknown_top_level_call = true;
        }
        self.unknown_calls.push(UnknownCall {
            caller,
            caller_id: self.current_function_id(),
            line,
            offset,
            invocation,
        });
    }
}
