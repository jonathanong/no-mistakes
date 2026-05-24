use crate::playwright::analysis::text_types::{
    AppTextKind, AppTextTarget, LocatorKind, PlaywrightTextLocator,
};
use std::collections::BTreeMap;

pub(super) struct AppTextIndex<'a> {
    exact: BTreeMap<AppTextIndexKey, Vec<&'a AppTextTarget>>,
    by_kind_role: BTreeMap<AppTextKindRoleKey, Vec<&'a AppTextTarget>>,
}

impl<'a> AppTextIndex<'a> {
    pub(super) fn new(targets: &'a [AppTextTarget]) -> Self {
        let mut exact: BTreeMap<AppTextIndexKey, Vec<&'a AppTextTarget>> = BTreeMap::new();
        let mut by_kind_role: BTreeMap<AppTextKindRoleKey, Vec<&'a AppTextTarget>> =
            BTreeMap::new();
        for target in targets {
            let kind_role = AppTextKindRoleKey::from_target(target);
            by_kind_role
                .entry(kind_role.clone())
                .or_default()
                .push(target);
            exact
                .entry(AppTextIndexKey {
                    kind_role,
                    text: target.text.clone(),
                })
                .or_default()
                .push(target);
        }
        Self {
            exact,
            by_kind_role,
        }
    }

    pub(super) fn candidates(&self, locator: &PlaywrightTextLocator) -> Vec<&'a AppTextTarget> {
        let Some(kind_role) = AppTextKindRoleKey::from_locator(locator) else {
            return Vec::new();
        };
        if locator.exact {
            return self
                .exact
                .get(&AppTextIndexKey {
                    kind_role,
                    text: locator.text.clone(),
                })
                .cloned()
                .unwrap_or_default();
        }
        self.by_kind_role
            .get(&kind_role)
            .cloned()
            .unwrap_or_default()
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
struct AppTextIndexKey {
    kind_role: AppTextKindRoleKey,
    text: String,
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
            LocatorKind::Role => AppTextKind::AccessibleName,
        };
        let role = match locator.kind {
            LocatorKind::Role => Some(locator.role.clone()?),
            _ => None,
        };
        Some(Self { kind, role })
    }
}
