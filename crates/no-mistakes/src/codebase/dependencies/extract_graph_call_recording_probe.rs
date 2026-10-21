thread_local! {
    static EXPRESSION_CALL_FACT_CONSTRUCTIONS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static UNKNOWN_CALL_FACT_CONSTRUCTIONS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

pub(crate) fn reset() {
    EXPRESSION_CALL_FACT_CONSTRUCTIONS.with(|count| count.set(0));
    UNKNOWN_CALL_FACT_CONSTRUCTIONS.with(|count| count.set(0));
}

pub(crate) fn counts() -> (u32, u32) {
    (
        EXPRESSION_CALL_FACT_CONSTRUCTIONS.with(std::cell::Cell::get),
        UNKNOWN_CALL_FACT_CONSTRUCTIONS.with(std::cell::Cell::get),
    )
}

pub(super) fn record_expression() {
    EXPRESSION_CALL_FACT_CONSTRUCTIONS.with(|count| count.set(count.get() + 1));
}

pub(super) fn record_unknown() {
    UNKNOWN_CALL_FACT_CONSTRUCTIONS.with(|count| count.set(count.get() + 1));
}

macro_rules! record_expression_call_fact {
    () => {
        graph_call_recording_probe::record_expression()
    };
}

macro_rules! record_unknown_call_fact {
    () => {
        graph_call_recording_probe::record_unknown()
    };
}
