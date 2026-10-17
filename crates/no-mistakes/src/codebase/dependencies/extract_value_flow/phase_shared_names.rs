use std::rc::Rc;

pub(in super::super) fn share_previous(previous: Option<&Rc<str>>, name: &str) -> Rc<str> {
    if let Some(previous) = previous.filter(|previous| previous.as_ref() == name) {
        return copy_name(previous);
    }
    new_name(name)
}

pub(in super::super) fn new_name(name: &str) -> Rc<str> {
    #[cfg(all(test, feature = "test-instrumentation"))]
    tests::MATERIALIZATIONS.with(|n| n.set(n.get() + 1));
    Rc::from(name)
}
pub(in super::super) fn copy_name(name: &Rc<str>) -> Rc<str> {
    #[cfg(all(test, feature = "test-instrumentation"))]
    if tests::OLD_COPY.with(std::cell::Cell::get) {
        return new_name(name);
    }
    Rc::clone(name)
}
#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "phase_shared_names_tests.rs"]
mod tests;
