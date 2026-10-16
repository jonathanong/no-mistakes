use super::*;

impl ParameterCache {
    #[cfg(feature = "test-instrumentation")]
    pub(in super::super::super) fn origin_nodes(&self) -> usize {
        self.origin_nodes.load(std::sync::atomic::Ordering::Relaxed)
    }

    #[cfg(feature = "test-instrumentation")]
    pub(in super::super) fn record_execution_step(&self) {
        self.execution_steps
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    #[cfg(feature = "test-instrumentation")]
    pub(in super::super::super) fn execution_steps(&self) -> usize {
        self.execution_steps
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}
