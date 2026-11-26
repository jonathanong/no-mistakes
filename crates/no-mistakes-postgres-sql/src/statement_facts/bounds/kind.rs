/// Which statement a bound fact judges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlBoundKind {
    Select,
    Update,
    Delete,
}
