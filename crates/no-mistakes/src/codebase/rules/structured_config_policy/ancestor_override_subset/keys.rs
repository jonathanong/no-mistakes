use crate::codebase::rules::structured_config_policy::ValueAssertion;

pub(super) struct Keys<'a> {
    pub(super) extends: &'a str,
    pub(super) overrides: &'a str,
    pub(super) files: &'a str,
    pub(super) exclude_files: &'a str,
    pub(super) rules: &'a str,
}

impl<'a> Keys<'a> {
    pub(super) fn from_assertion(assertion: &'a ValueAssertion) -> Self {
        Self {
            extends: named(&assertion.extends_key, "extends"),
            overrides: named(&assertion.overrides_key, "overrides"),
            files: named(&assertion.files_key, "files"),
            exclude_files: named(&assertion.exclude_files_key, "excludeFiles"),
            rules: named(&assertion.rules_key, "rules"),
        }
    }
}

fn named<'a>(value: &'a str, default: &'a str) -> &'a str {
    if value.is_empty() {
        default
    } else {
        value
    }
}
