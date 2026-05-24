use crate::playwright::analysis::text_types::{AppTextKind, AppTextTarget, LocatorKind};

pub(super) struct TextMatch<'a> {
    exact: bool,
    locator: &'a str,
    locator_lower: Option<String>,
}

impl<'a> TextMatch<'a> {
    pub(super) fn new(locator: &'a str, exact: bool) -> Self {
        Self {
            exact,
            locator,
            locator_lower: (!exact).then(|| locator.to_lowercase()),
        }
    }

    fn matches(&self, target: &str) -> bool {
        if self.exact {
            return target == self.locator;
        }
        target
            .to_lowercase()
            .contains(self.locator_lower.as_deref().unwrap_or(self.locator))
    }
}

pub(super) fn text_target_matches(
    target: &AppTextTarget,
    kind: &LocatorKind,
    role: Option<&str>,
    text: &TextMatch<'_>,
    include_hidden: bool,
) -> bool {
    text.matches(&target.text)
        && match kind {
            LocatorKind::Text => target.kind == AppTextKind::VisibleText,
            LocatorKind::Label => target.kind == AppTextKind::Label,
            LocatorKind::Placeholder => target.kind == AppTextKind::Placeholder,
            LocatorKind::Role => {
                target.role.as_deref() == role
                    && (include_hidden || !target.hidden)
                    && target.kind == AppTextKind::AccessibleName
            }
        }
}

#[cfg(test)]
mod tests;
