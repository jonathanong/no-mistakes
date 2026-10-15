use super::ParameterCache;

impl ParameterCache {
    #[cfg(feature = "test-instrumentation")]
    pub(in super::super) fn record_nullish_work(&self, slot: usize, count: usize) {
        self.nullish_work[slot].fetch_add(count, std::sync::atomic::Ordering::Relaxed);
    }
    #[cfg(feature = "test-instrumentation")]
    pub(in super::super) fn record_nullish_peak(&self, slot: usize, count: usize) {
        self.nullish_work[slot].fetch_max(count, std::sync::atomic::Ordering::Relaxed);
    }
    #[cfg(feature = "test-instrumentation")]
    pub(in super::super::super) fn nullish_work(&self) -> [usize; 11] {
        self.nullish_work
            .each_ref()
            .map(|count| count.load(std::sync::atomic::Ordering::Relaxed))
    }
}
