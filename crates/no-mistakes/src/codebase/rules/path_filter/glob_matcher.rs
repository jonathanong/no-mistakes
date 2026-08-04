use anyhow::Result;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

pub(crate) struct GlobMatcher {
    globset: Option<GlobSet>,
}

impl GlobMatcher {
    pub(crate) fn new(patterns: &[String], context: &str) -> Result<Self> {
        let mut builder = GlobSetBuilder::new();
        let mut count = 0usize;
        for pattern in patterns {
            let normalized = pattern.trim_start_matches("./");
            let glob_result = GlobBuilder::new(normalized)
                .literal_separator(false)
                .build();
            let glob = match glob_result {
                Ok(glob) => glob,
                Err(error) => {
                    return Err(anyhow::Error::new(error)
                        .context(format!("{context} contains invalid glob `{pattern}`")));
                }
            };
            builder.add(glob);
            count += 1;
        }
        let globset = if count == 0 {
            None
        } else {
            let result = builder.build();
            let globset = match result {
                Ok(globset) => globset,
                Err(error) => {
                    return Err(anyhow::Error::new(error)
                        .context(format!("failed to build {context} glob set")));
                }
            };
            Some(globset)
        };
        Ok(Self { globset })
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.globset.is_none()
    }

    pub(crate) fn is_match(&self, rel: &str) -> bool {
        self.globset
            .as_ref()
            .is_some_and(|globset| globset.is_match(rel))
    }
}
