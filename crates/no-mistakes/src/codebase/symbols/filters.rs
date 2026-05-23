pub(crate) struct KindFilter {
    allowed: std::collections::HashSet<ExportKindArg>,
}

impl KindFilter {
    fn matches_export(&self, k: &ExportKind) -> bool {
        self.allowed.iter().any(|arg| arg.matches(k))
    }
}

fn build_kind_filter(kinds: &[ExportKindArg]) -> Option<KindFilter> {
    if kinds.is_empty() {
        return None;
    }
    Some(KindFilter {
        allowed: kinds.iter().copied().collect(),
    })
}

/// Stable string name for an `ExportKind` — used as the `kind` field in JSON output.
pub fn export_kind_str(k: &ExportKind) -> &'static str {
    match k {
        ExportKind::Function => "function",
        ExportKind::Class => "class",
        ExportKind::Const => "const",
        ExportKind::Let => "let",
        ExportKind::Var => "var",
        ExportKind::TypeAlias => "type",
        ExportKind::Interface => "interface",
        ExportKind::Enum => "enum",
        ExportKind::Default => "default",
        ExportKind::ReExport { .. } => "re-export",
    }
}

