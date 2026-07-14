/// Adds facts for files missing from a caller-provided sparse lookup while
/// preserving per-file Playwright facts. App-wide memoization is safe only
/// when the primary lookup and this graph describe the same file universe.
struct FallbackTsFactLookup<'a> {
    primary: &'a dyn TsFactLookup,
    fallback: &'a TsFactMap,
    prefer_fallback: bool,
    graph_files: &'a [PathBuf],
    reuse_primary_playwright_cache: bool,
}

impl<'a> FallbackTsFactLookup<'a> {
    fn new(
        primary: &'a dyn TsFactLookup,
        fallback: &'a TsFactMap,
        prefer_fallback: bool,
        graph_files: &'a [PathBuf],
        graph_visible: &HashSet<PathBuf>,
    ) -> Self {
        let reuse_primary_playwright_cache = primary
            .graph_files()
            .is_some_and(|primary_files| same_graph_universe(primary_files, graph_visible));
        Self {
            primary,
            fallback,
            prefer_fallback,
            graph_files,
            reuse_primary_playwright_cache,
        }
    }
}

fn same_graph_universe(primary_files: &[PathBuf], graph_visible: &HashSet<PathBuf>) -> bool {
    let primary_visible: HashSet<&Path> = primary_files.iter().map(PathBuf::as_path).collect();
    primary_visible.len() == graph_visible.len()
        && primary_visible
            .iter()
            .all(|path| graph_visible.contains(*path))
}

impl TsFactLookup for FallbackTsFactLookup<'_> {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
        if self.prefer_fallback {
            self.fallback
                .get(path)
                .or_else(|| self.primary.get_ts_facts(path))
        } else {
            self.primary
                .get_ts_facts(path)
                .or_else(|| self.fallback.get(path))
        }
    }

    fn covers_ts_fact_plan(&self, _required: TsFactPlan) -> bool {
        true
    }

    fn graph_files(&self) -> Option<&[PathBuf]> {
        Some(self.graph_files)
    }

    fn get_playwright_facts(
        &self,
        path: &Path,
    ) -> Option<&crate::codebase::check_facts::PlaywrightTestFacts> {
        self.primary.get_playwright_facts(path)
    }

    fn get_playwright_parse_error(&self, path: &Path) -> Option<&str> {
        if self.prefer_fallback {
            self.fallback
                .get(path)
                .and_then(|facts| facts.parse_error.as_deref())
                .or_else(|| self.primary.get_playwright_parse_error(path))
        } else {
            self.primary.get_playwright_parse_error(path).or_else(|| {
                self.fallback
                    .get(path)
                    .and_then(|facts| facts.parse_error.as_deref())
            })
        }
    }

    fn playwright_source_files(&self) -> Option<&[PathBuf]> {
        if self.reuse_primary_playwright_cache {
            self.primary.playwright_source_files()
        } else {
            None
        }
    }

    fn get_playwright_test_files(
        &self,
        project: Option<&str>,
    ) -> Option<Arc<Vec<crate::playwright::analysis::context::DiscoveredTestFile>>> {
        if self.reuse_primary_playwright_cache {
            self.primary.get_playwright_test_files(project)
        } else {
            None
        }
    }

    fn get_playwright_fetch_facts(
        &self,
        path: &Path,
    ) -> Option<Result<crate::fetch::file_facts::ParsedFileFacts, String>> {
        if !self.reuse_primary_playwright_cache {
            return None;
        }
        let fallback_error = || {
            self.fallback.get(path).and_then(|facts| {
                facts.parse_error.as_ref().map(|error| {
                    Err(format!("failed to parse {}: {error}", path.display()))
                })
            })
        };
        if self.prefer_fallback {
            fallback_error().or_else(|| self.primary.get_playwright_fetch_facts(path))
        } else {
            self.primary
                .get_playwright_fetch_facts(path)
                .or_else(fallback_error)
        }
    }

    fn get_or_compute_app_selector_occurrences(
        &self,
        settings: &crate::playwright::config::Settings,
        scan_html_ids: bool,
        compute: &dyn Fn() -> Result<Vec<crate::playwright::selectors::AppSelector>>,
    ) -> Result<Arc<Vec<crate::playwright::selectors::AppSelector>>> {
        if self.reuse_primary_playwright_cache {
            self.primary
                .get_or_compute_app_selector_occurrences(settings, scan_html_ids, compute)
        } else {
            compute().map(Arc::new)
        }
    }

    fn get_or_compute_playwright_routes(
        &self,
        settings: &crate::playwright::config::Settings,
        compute: &dyn Fn() -> Vec<crate::routes::Route>,
    ) -> Arc<Vec<crate::routes::Route>> {
        if self.reuse_primary_playwright_cache {
            self.primary.get_or_compute_playwright_routes(settings, compute)
        } else {
            Arc::new(compute())
        }
    }

    fn get_or_compute_app_text_targets(
        &self,
        settings: &crate::playwright::config::Settings,
        compute: &dyn Fn() -> Result<Vec<crate::playwright::analysis::text_types::AppTextTarget>>,
    ) -> Result<Arc<Vec<crate::playwright::analysis::text_types::AppTextTarget>>> {
        if self.reuse_primary_playwright_cache {
            self.primary.get_or_compute_app_text_targets(settings, compute)
        } else {
            compute().map(Arc::new)
        }
    }

    fn get_or_compute_route_reachable_files(
        &self,
        settings: &crate::playwright::config::Settings,
        compute: &dyn Fn() -> Result<RouteReachableFiles>,
    ) -> Result<Arc<RouteReachableFiles>> {
        if self.reuse_primary_playwright_cache {
            self.primary
                .get_or_compute_route_reachable_files(settings, compute)
        } else {
            compute().map(Arc::new)
        }
    }
}
