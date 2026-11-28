use std::sync::Arc;

pub(crate) mod playwright_provenance;

pub(crate) struct Location {
    pub(crate) shape: Shape,
    pub(crate) line: usize,
    pub(crate) project: Option<String>,
    pub(crate) origin: Origin,
    pub(crate) playwright_candidates: Vec<playwright_provenance::ImportRef>,
    pub(crate) playwright_import: Option<playwright_provenance::ImportRef>,
    pub(crate) playwright_binding: Option<Arc<str>>,
    pub(crate) playwright_wrappers: Vec<playwright_provenance::WrapperCall>,
    pub(crate) malformed_playwright_fixture: Option<String>,
    pub(crate) resolved_playwright_exports: Vec<(String, Arc<str>)>,
    pub(crate) unresolved_reason: Option<&'static str>,
}

pub(crate) enum Shape {
    Retry(&'static str, Option<u64>),
    Serial(&'static str),
}

#[derive(Clone, Copy, Default)]
pub struct SourceDemand {
    pub(crate) vitest_config: bool,
    pub(crate) playwright_config: bool,
    pub(crate) playwright_factory: bool,
    pub(crate) standard_tests: bool,
    pub(crate) playwright_tests: bool,
    pub(crate) vitest_globals: bool,
}

impl SourceDemand {
    #[doc(hidden)]
    pub fn includes_playwright_tests(self) -> bool {
        self.playwright_tests
    }

    pub(crate) fn include(&mut self, other: Self) {
        self.vitest_config |= other.vitest_config;
        self.playwright_config |= other.playwright_config;
        self.playwright_factory |= other.playwright_factory;
        self.standard_tests |= other.standard_tests;
        self.playwright_tests |= other.playwright_tests;
        self.vitest_globals |= other.vitest_globals;
    }
    pub(crate) fn includes(self, origin: Origin) -> bool {
        match origin {
            Origin::Settings => true,
            Origin::PlaywrightFactory => self.playwright_factory,
            Origin::Standard => self.standard_tests,
            Origin::VitestGlobal => self.standard_tests && self.vitest_globals,
            Origin::Playwright => self.playwright_tests,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Origin {
    Settings,
    PlaywrightFactory,
    Standard,
    VitestGlobal,
    Playwright,
}
