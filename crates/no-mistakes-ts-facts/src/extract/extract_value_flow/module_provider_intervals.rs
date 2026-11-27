/// Shared possible module-provider intervals for one projected receiver target.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleProviderTarget {
    pub intervals: Vec<(u32, u32)>,
    pub uncertain: bool,
}
impl ModuleProviderTarget {
    pub fn contains(&self, phase: u32) -> bool {
        if self.uncertain {
            return true;
        }
        let end = self.intervals.partition_point(|(start, _)| *start <= phase);
        end.checked_sub(1)
            .is_some_and(|index| phase < self.intervals[index].1)
    }
    pub(super) fn finish(&mut self) {
        self.intervals.sort_unstable();
        let mut retained = 0;
        for index in 0..self.intervals.len() {
            let interval = self.intervals[index];
            if retained > 0 && interval.0 <= self.intervals[retained - 1].1 {
                self.intervals[retained - 1].1 = self.intervals[retained - 1].1.max(interval.1);
            } else {
                self.intervals[retained] = interval;
                retained += 1;
            }
        }
        self.intervals.truncate(retained);
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "module_provider_intervals_tests.rs"]
mod tests;
