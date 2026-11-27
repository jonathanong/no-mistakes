use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;

#[derive(Clone)]
pub struct TestFilter {
    pub(crate) include: GlobSet,
    pub(crate) include_regex: Vec<Regex>,
    pub(crate) exclude: GlobSet,
    pub(crate) required: Option<GlobSet>,
    pub(crate) alternatives: Vec<TestFilter>,
}

impl TestFilter {
    pub fn is_match(&self, rel_path: &str) -> bool {
        let included = self
            .alternatives
            .iter()
            .any(|filter| filter.is_match(rel_path))
            || self.include.is_match(rel_path)
            || self
                .include_regex
                .iter()
                .any(|regex| regex.is_match(rel_path));
        included
            && !self.exclude.is_match(rel_path)
            && self
                .required
                .as_ref()
                .is_none_or(|scope| scope.is_match(rel_path))
    }
}

impl TestFilter {
    pub(crate) fn union(filters: Vec<Self>, excludes: &[String]) -> Result<Self> {
        Ok(Self {
            include: build_globset(&[])?,
            include_regex: Vec::new(),
            exclude: build_globset(excludes)?,
            required: None,
            alternatives: filters,
        })
    }
}

pub(crate) fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(builder.build()?)
}
