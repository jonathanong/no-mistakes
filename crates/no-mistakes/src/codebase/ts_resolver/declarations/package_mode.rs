use std::path::Path;

/// For static imports/reexports, explicit extensions prove a Node condition.
/// Ambiguous TS/JS files retain the all-branches declaration agreement rule.
#[derive(Clone, Copy)]
pub(super) enum ConditionMode {
    Import,
    Require,
    Unknown,
}

impl ConditionMode {
    pub(super) fn for_importer(importer: &Path) -> Self {
        match importer
            .extension()
            .and_then(|extension| extension.to_str())
        {
            Some("mts" | "mjs") => Self::Import,
            Some("cts" | "cjs") => Self::Require,
            _ => Self::Unknown,
        }
    }

    pub(super) fn applies(self, condition: &str) -> Option<bool> {
        match condition {
            "default" => Some(true),
            "import" => match self {
                Self::Import => Some(true),
                Self::Require => Some(false),
                Self::Unknown => None,
            },
            "require" => self.applies("import").map(|import| !import),
            _ => None,
        }
    }
}
