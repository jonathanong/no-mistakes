use crate::playwright::analysis::types::SelectorRef;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum AppTextKind {
    VisibleText,
    Label,
    Placeholder,
    AccessibleName,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct AppTextTarget {
    pub(crate) file: PathBuf,
    pub(crate) app_file: Arc<String>,
    pub(crate) kind: AppTextKind,
    pub(crate) text: String,
    pub(crate) selector_refs: Vec<SelectorRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum LocatorKind {
    Role,
    Text,
    Label,
    Placeholder,
}

impl LocatorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Role => "role",
            Self::Text => "text",
            Self::Label => "label",
            Self::Placeholder => "placeholder",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct PlaywrightTextLocator {
    pub(crate) kind: LocatorKind,
    pub(crate) role: Option<String>,
    pub(crate) text: String,
    pub(crate) locator: String,
}

pub(crate) fn normalize_locator_text(value: &str) -> Option<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    (!normalized.is_empty()).then_some(normalized)
}
