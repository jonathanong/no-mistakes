use serde::{Deserialize, Serialize};

/// Opt-in supplemental route attribution. This never contributes selector coverage.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RouteCoverageSource {
    pub framework: RouteCoverageFramework,
    /// The actual runner project name, not a Playwright project.
    pub project: String,
    /// Candidate modules, including statically imported test-registration modules.
    pub include: Vec<String>,
    /// Exact canonical route identifiers; named parameters allowed, no exemptions.
    pub routes: Vec<String>,
    pub helpers: Vec<RouteCoverageHelper>,
}

impl RouteCoverageSource {
    pub(crate) fn empty_required_field(&self) -> Option<&'static str> {
        [
            ("project", self.project.is_empty()),
            ("include", self.include.is_empty()),
            ("routes", self.routes.is_empty()),
            ("helpers", self.helpers.is_empty()),
        ]
        .into_iter()
        .find_map(|(field, empty)| empty.then_some(field))
    }

    pub(crate) fn first_noncanonical_route(&self) -> Option<(usize, &str)> {
        self.routes
            .iter()
            .map(String::as_str)
            .enumerate()
            .find(|(_, route)| {
                !route.starts_with('/')
                    || route.starts_with("//")
                    || route.contains(['*', '[', ']', '{', '}', '?', '#'])
            })
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub enum RouteCoverageFramework {
    Vitest,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RouteCoverageHelper {
    /// Exact repository-relative file or import-resolvable module specifier.
    pub module: String,
    pub export: String,
    /// Omit for an imported function; set for an imported class instance method.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    pub url_argument: usize,
}

impl RouteCoverageHelper {
    pub(crate) fn empty_field(&self) -> Option<&'static str> {
        [
            ("module", self.module.is_empty()),
            ("export", self.export.is_empty()),
            ("method", self.method.as_ref().is_some_and(String::is_empty)),
        ]
        .into_iter()
        .find_map(|(field, empty)| empty.then_some(field))
    }
}
