use super::*;

pub(in super::super) struct Capture;
impl Capture {
    pub(in super::super) fn start() -> Self {
        CAPTURE.with(|enabled| enabled.set(true));
        Self
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        CAPTURE.with(|enabled| enabled.set(false));
        CANDIDATES.with(|values| values.borrow_mut().clear());
    }
}
