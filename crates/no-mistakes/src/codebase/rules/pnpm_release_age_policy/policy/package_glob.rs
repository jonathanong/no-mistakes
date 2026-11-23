use regex::{Regex, RegexBuilder};

pub(super) struct PackageGlob {
    pattern: Regex,
    negated: bool,
}

impl PackageGlob {
    pub(super) fn is_glob(value: &str) -> bool {
        value.starts_with('!') || value.contains(['*', '?'])
    }

    pub(super) fn new(value: &str) -> Option<Self> {
        let name = value.strip_prefix('!').unwrap_or(value);
        if name.trim().is_empty() || name.strip_prefix('@').unwrap_or(name).contains('@') {
            return None;
        }
        let pattern = regex::escape(name).replace(r"\*", ".*").replace(r"\?", ".");
        let pattern = RegexBuilder::new(&format!(r"\A{pattern}\z"))
            .dot_matches_new_line(true)
            .build()
            .ok()?;
        Some(Self {
            pattern,
            negated: value.starts_with('!'),
        })
    }

    pub(super) fn matches(&self, name: &str) -> bool {
        self.pattern.is_match(name) != self.negated
    }
}
