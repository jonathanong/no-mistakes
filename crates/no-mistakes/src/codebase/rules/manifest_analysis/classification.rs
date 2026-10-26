use super::RuleFinding;

/// Internal provenance is independent of public, user-derived target values.
#[derive(Debug)]
pub(crate) enum ClassifiedFinding {
    Definite(RuleFinding),
    Unanalyzable(RuleFinding),
}

impl ClassifiedFinding {
    pub(crate) fn finding(&self) -> &RuleFinding {
        match self {
            Self::Definite(finding) | Self::Unanalyzable(finding) => finding,
        }
    }

    pub(crate) fn into_public(self) -> RuleFinding {
        match self {
            Self::Definite(finding) | Self::Unanalyzable(finding) => finding,
        }
    }
}
