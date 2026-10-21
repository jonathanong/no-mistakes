//! Production hook has no state, allocation, clock or observer work.
pub(super) struct Hook;
#[inline(always)]
pub(super) fn current() -> Hook {
    Hook
}
impl Hook {
    #[inline(always)]
    pub fn enter(&self) {}
}
