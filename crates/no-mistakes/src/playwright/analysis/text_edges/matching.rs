use crate::playwright::analysis::text_types::{AppTextKind, AppTextTarget, LocatorKind};

pub(super) fn text_target_matches(
    target: &AppTextTarget,
    kind: &LocatorKind,
    role: Option<&str>,
    text: &str,
    exact: bool,
    include_hidden: bool,
) -> bool {
    locator_text_matches(&target.text, text, exact)
        && match kind {
            LocatorKind::Text => target.kind == AppTextKind::VisibleText,
            LocatorKind::Label => target.kind == AppTextKind::Label,
            LocatorKind::Placeholder => target.kind == AppTextKind::Placeholder,
            LocatorKind::Role => {
                target.role.as_deref() == role
                    && (include_hidden || !target.hidden)
                    && (target.kind == AppTextKind::VisibleText
                        || target.kind == AppTextKind::AccessibleName)
            }
        }
}

fn locator_text_matches(target: &str, locator: &str, exact: bool) -> bool {
    if exact {
        return target == locator;
    }
    target.to_lowercase().contains(&locator.to_lowercase())
}
