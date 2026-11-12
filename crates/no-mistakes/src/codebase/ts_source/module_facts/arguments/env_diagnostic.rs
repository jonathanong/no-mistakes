#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnvDiagnostic {
    pub(crate) name: String,
    pub(crate) fallback: Option<String>,
    pub(crate) captured: bool,
    pub(crate) line: u32,
}
