//! Per-root semantic limits, independent of memoization and worker scheduling.
#[derive(Clone, Copy)]
pub(super) struct Limits {
    pub depth: usize,
    pub variants: usize,
    pub work: usize,
    pub sql_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            depth: 8,
            variants: 16,
            work: 10_000,
            sql_bytes: 1_048_576,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Exhausted {
    Depth,
    Variants,
    Work,
    SqlBytes,
}
#[derive(Clone, Copy, Default)]
pub(super) struct Cost {
    pub work: usize,
    pub depth: usize,
    pub sql_bytes: usize,
}
pub(super) struct Budget {
    limits: Limits,
    used: usize,
    bytes: usize,
}
impl Budget {
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            used: 0,
            bytes: 0,
        }
    }
    /// Cached summaries pay their intrinsic semantic cost too. Otherwise cache
    /// warmth could decide whether the same root silently exceeds its limits.
    pub fn charge(&mut self, cost: Cost, caller_depth: usize) -> Result<(), Exhausted> {
        if caller_depth
            .checked_add(cost.depth)
            .is_none_or(|depth| depth > self.limits.depth)
        {
            return Err(Exhausted::Depth);
        }
        let Some(next) = self
            .used
            .checked_add(cost.work)
            .filter(|used| *used <= self.limits.work)
        else {
            return Err(Exhausted::Work);
        };
        let bytes = self
            .bytes
            .checked_add(cost.sql_bytes)
            .filter(|bytes| *bytes <= self.limits.sql_bytes)
            .ok_or(Exhausted::SqlBytes)?;
        self.bytes = bytes;
        self.used = next;
        Ok(())
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }
    pub fn used(&self) -> usize {
        self.used
    }
    pub fn max_variants(&self) -> usize {
        self.limits.variants
    }
    pub fn variants(&self, count: usize) -> Result<(), Exhausted> {
        if count > self.limits.variants {
            Err(Exhausted::Variants)
        } else {
            Ok(())
        }
    }
}
