use crate::playwright::analysis::text_types::{
    AppTextKind, AppTextTarget, LocatorKind, PlaywrightTextLocator,
};
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct AppTextIndex {
    exact: BTreeMap<AppTextKindRoleKey, BTreeMap<String, Vec<usize>>>,
    by_kind_role: BTreeMap<AppTextKindRoleKey, Vec<usize>>,
}

impl AppTextIndex {
    pub(crate) fn new(targets: &[AppTextTarget]) -> Self {
        let mut exact: BTreeMap<AppTextKindRoleKey, BTreeMap<String, Vec<usize>>> = BTreeMap::new();
        let mut by_kind_role: BTreeMap<AppTextKindRoleKey, Vec<usize>> = BTreeMap::new();
        for (position, target) in targets.iter().enumerate() {
            let kind_role = AppTextKindRoleKey::from_target(target);
            by_kind_role
                .entry(kind_role.clone())
                .or_default()
                .push(position);
            exact
                .entry(kind_role)
                .or_default()
                .entry(target.text.clone())
                .or_default()
                .push(position);
        }
        Self {
            exact,
            by_kind_role,
        }
    }

    pub(crate) fn candidates(&self, locator: &PlaywrightTextLocator) -> &[usize] {
        let Some(kind_role) = AppTextKindRoleKey::from_locator(locator) else {
            return &[];
        };
        if locator.exact {
            return self
                .exact
                .get(&kind_role)
                .and_then(|by_text| by_text.get(locator.text.as_str()))
                .map(Vec::as_slice)
                .unwrap_or_default();
        }
        self.by_kind_role
            .get(&kind_role)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
struct AppTextKindRoleKey {
    kind: AppTextKind,
    role: Option<String>,
}

impl AppTextKindRoleKey {
    fn from_target(target: &AppTextTarget) -> Self {
        let role = if target.kind == AppTextKind::AccessibleName {
            target.role.clone()
        } else {
            None
        };
        Self {
            kind: target.kind.clone(),
            role,
        }
    }

    fn from_locator(locator: &PlaywrightTextLocator) -> Option<Self> {
        let kind = match locator.kind {
            LocatorKind::Text => AppTextKind::VisibleText,
            LocatorKind::Label => AppTextKind::Label,
            LocatorKind::Placeholder => AppTextKind::Placeholder,
            LocatorKind::Alt => AppTextKind::Alt,
            LocatorKind::Title => AppTextKind::Title,
            LocatorKind::Role => AppTextKind::AccessibleName,
        };
        let role = match locator.kind {
            LocatorKind::Role => Some(locator.role.clone()?),
            _ => None,
        };
        Some(Self { kind, role })
    }
}
