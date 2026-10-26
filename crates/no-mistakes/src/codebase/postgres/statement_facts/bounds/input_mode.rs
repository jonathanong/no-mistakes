/// Whether set-operation inputs stream, must complete before output, or cannot execute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SqlBoundInputMode {
    #[default]
    Streaming,
    Blocking,
    Skipped,
}
