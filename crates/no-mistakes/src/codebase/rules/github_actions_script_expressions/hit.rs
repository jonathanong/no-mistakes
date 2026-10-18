#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct Hit {
    pub(super) line: usize,
    pub(super) expr: String,
    pub(super) script: bool,
}
