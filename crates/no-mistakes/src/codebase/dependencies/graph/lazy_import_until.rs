/// Terminal files for a lazy import walk. Matching files are in the graph
/// as sinks: they are not parsed and their imports are not followed.
pub(crate) struct UntilMatcher {
    set: Option<GlobSet>,
}

impl UntilMatcher {
    pub(crate) fn parse(patterns: &[String]) -> anyhow::Result<Self> {
        if patterns.is_empty() {
            return Ok(Self { set: None });
        }
        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            builder.add(Glob::new(pattern)?);
        }
        Ok(Self {
            set: Some(builder.build()?),
        })
    }

    pub(crate) fn matches(&self, root: &Path, path: &Path) -> bool {
        let Some(set) = &self.set else {
            return false;
        };
        set.is_match(crate::codebase::ts_source::relative_slash_path(root, path))
    }
}
