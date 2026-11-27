#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvDiagnostic {
    pub name: String,
    pub fallback: Option<String>,
    pub captured: bool,
    pub line: u32,
}
