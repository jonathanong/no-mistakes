use crate::codebase::dependencies::extract::{is_indexable, ExtractedImport};
use crate::codebase::rules::nextjs_no_caching::NextjsCachingFinding;
use crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::TestFacts;
use crate::codebase::storybook::StorybookFileFacts;
use crate::codebase::ts_symbols::FileSymbols;
use crate::integration_tests::types::FileAnalysis as IntegrationFileAnalysis;
use crate::playwright::analysis::text_types::PlaywrightTextLocator;
use crate::playwright::playwright_tests::TestOccurrence;
use crate::playwright::selectors::{PlaywrightSelector, SelectorRegexes};
use crate::queue::extract::FileFacts as QueueFileFacts;
use crate::react_traits::analyze::file::FileAnalysis as ReactFileAnalysis;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod file;
pub(crate) use file::collect_file_facts;

#[derive(Clone, Default)]
pub struct CheckFactPlan {
    pub imports: bool,
    pub symbols: bool,
    pub react: bool,
    pub queue: bool,
    pub queue_factory_names: Vec<String>,
    pub integration: bool,
    pub dynamic_imports: bool,
    pub nextjs_caching: bool,
    pub storybook: bool,
    pub source: bool,
    pub raw_source: bool,
}

#[derive(Clone)]
pub struct PlaywrightFactPlan {
    pub(crate) navigation_helpers: Vec<String>,
    pub(crate) selector_regexes: Arc<SelectorRegexes>,
    pub(crate) test_id_attributes_by_path: Arc<HashMap<PathBuf, Vec<String>>>,
}

#[derive(Default)]
pub struct CheckFactMap {
    pub(crate) files: Vec<PathBuf>,
    pub(crate) ts: HashMap<PathBuf, CheckFileFacts>,
    pub stats: CheckFactStats,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CheckFactStats {
    pub files_discovered: usize,
    pub files_parsed: usize,
    pub parse_errors: usize,
}

#[derive(Default)]
pub(crate) struct CheckFileFacts {
    pub source: Option<String>,
    pub imports: Vec<ExtractedImport>,
    pub symbols: Option<FileSymbols>,
    pub react: Option<ReactFileAnalysis>,
    pub queue: Option<QueueFileFacts>,
    pub integration: Option<IntegrationFileAnalysis>,
    pub dynamic_imports: Option<TestFacts>,
    pub nextjs_caching: Option<Vec<NextjsCachingFinding>>,
    pub storybook: Option<StorybookFileFacts>,
    pub(crate) playwright: Option<PlaywrightTestFacts>,
    pub parse_error: Option<String>,
    pub(crate) parsed: bool,
}

pub(crate) struct PlaywrightTestFacts {
    pub(crate) urls: Vec<TestOccurrence<String>>,
    pub(crate) selectors: Vec<TestOccurrence<PlaywrightSelector>>,
    pub(crate) text_locators: Vec<TestOccurrence<PlaywrightTextLocator>>,
}

impl CheckFactMap {
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    pub(crate) fn ts_facts(&self) -> crate::codebase::ts_source::facts::TsFactMap {
        let mut ts_facts = crate::codebase::ts_source::facts::TsFactMap::new();
        for (path, facts) in &self.ts {
            ts_facts.insert(
                path.clone(),
                crate::codebase::ts_source::facts::TsFileFacts {
                    source: facts.source.clone(),
                    imports: facts.imports.clone(),
                    symbols: facts.symbols.clone(),
                    ..Default::default()
                },
            );
        }
        ts_facts
    }
}

pub fn collect_check_facts(root: &Path, files: Vec<PathBuf>, plan: CheckFactPlan) -> CheckFactMap {
    collect_check_facts_with_playwright(root, files, plan, None)
}

pub fn collect_check_facts_with_playwright(
    root: &Path,
    files: Vec<PathBuf>,
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
) -> CheckFactMap {
    let stats = CheckFactStats {
        files_discovered: files.len(),
        ..CheckFactStats::default()
    };
    let playwright = playwright.as_ref();
    let ts: HashMap<_, _> = files
        .par_iter()
        .filter(|path| is_indexable(path) || (plan.storybook && is_mdx_file(path)))
        .filter_map(|path| {
            collect_file_facts(root, path, &plan, playwright).map(|facts| (path.clone(), facts))
        })
        .collect();
    let mut files_parsed = 0;
    let mut parse_errors = 0;
    for facts in ts.values() {
        if facts.parsed {
            files_parsed += 1;
        }
        if facts.parse_error.is_some() {
            parse_errors += 1;
        }
    }
    CheckFactMap {
        files,
        ts,
        stats: CheckFactStats {
            files_parsed,
            parse_errors,
            ..stats
        },
    }
}

fn is_mdx_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("mdx")
}

#[cfg(test)]
mod tests;
