//! Why a parameter root stayed unproven. Acceptance stays in the walk.
use crate::codebase::dependencies::graph::{NodeId, TsFactLookup};
use crate::codebase::ts_source::relative_slash_path;
use std::path::{Path, PathBuf};

pub(in super::super) enum Ownership {
    Yes,
    No,
    Parameter(Cause),
}

pub(in super::super) struct Cause {
    pub(super) kind: CauseKind,
    file: PathBuf,
    line: u32,
    expression: String,
}

pub(super) enum CauseKind {
    LivePath,
    OmittedArgument,
    UnprovenVariable,
    ForwardedParameter,
    NamespaceImport,
    DefaultImport,
    DynamicImport,
    RequireImport,
    ViMock,
    NonDirectReference,
    NotNamedFunction,
    ReassignedParameter,
    NoCaller,
    ClimbsOut,
    UnresolvedSetup,
    UnreadModule,
}

impl Cause {
    pub(super) fn new(
        kind: CauseKind,
        file: PathBuf,
        line: u32,
        expression: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            file,
            line,
            expression: expression.into(),
        }
    }

    pub(in super::super) fn detail(&self, root: &Path) -> String {
        let place = format!("{}:{}", relative_slash_path(root, &self.file), self.line);
        let expr = self.expression.as_str();
        match self.kind {
            CauseKind::LivePath => passed(&place, expr, "a live path"),
            CauseKind::OmittedArgument => format!("{place} omits the root argument"),
            CauseKind::UnprovenVariable => passed(&place, expr, "an unproven variable"),
            CauseKind::ForwardedParameter => {
                passed(&place, expr, "a value forwarded through another parameter")
            }
            CauseKind::NamespaceImport => route(&place, expr, "a namespace import"),
            CauseKind::DefaultImport => route(&place, expr, "a default import"),
            CauseKind::DynamicImport => route(&place, expr, "a dynamic import()"),
            CauseKind::RequireImport => route(&place, expr, "a require()"),
            CauseKind::ViMock => route(&place, expr, "vi.mock"),
            CauseKind::NonDirectReference => format!(
                "callers are unknowable because {place} references `{expr}` other than as a direct call"
            ),
            CauseKind::NotNamedFunction => {
                format!("callers are unknowable because {place} is not a named function")
            }
            CauseKind::ReassignedParameter => format!("{place} reassigns the parameter"),
            CauseKind::NoCaller => {
                format!("no caller call site passes the parameter ({place} declares it)")
            }
            CauseKind::ClimbsOut => format!("{place} climbs above the caller-supplied root"),
            CauseKind::UnresolvedSetup => format!(
                "callers are unknowable because {place} does not resolve setupFiles"
            ),
            CauseKind::UnreadModule => format!(
                "callers are unknowable because {place} loads a module this analysis cannot read"
            ),
        }
    }

    pub(in super::super) fn fix(&self) -> &'static str {
        match self.kind {
            CauseKind::LivePath | CauseKind::OmittedArgument | CauseKind::UnprovenVariable => {
                "pass a const fixture path or an owned mkdtemp root from that caller"
            }
            CauseKind::ForwardedParameter => {
                "pass an owned root from the caller that supplies that parameter"
            }
            CauseKind::NamespaceImport
            | CauseKind::DefaultImport
            | CauseKind::DynamicImport
            | CauseKind::RequireImport => {
                "import the helper with a named static import and pass an owned root from the caller"
            }
            CauseKind::ViMock => "do not vi.mock this module, and pass an owned root from the caller",
            CauseKind::NonDirectReference => {
                "call the helper directly and pass an owned root from the caller"
            }
            CauseKind::NotNamedFunction => {
                "declare the helper as a named function and pass an owned root from each caller"
            }
            CauseKind::ReassignedParameter => {
                "stop reassigning the parameter and pass an owned root from each caller"
            }
            CauseKind::NoCaller => "pass an owned root from the caller",
            CauseKind::ClimbsOut => {
                "do not traverse above the caller-supplied root; pass an owned root from the caller"
            }
            CauseKind::UnresolvedSetup => {
                "give setupFiles a literal path, or pass an owned root from a caller this analysis can see"
            }
            CauseKind::UnreadModule => {
                "use a module specifier this analysis can read, and pass an owned root from the caller"
            }
        }
    }
}

fn passed(place: &str, expr: &str, why: &str) -> String {
    if expr.is_empty() {
        format!("{place} passes an argument that is {why}")
    } else {
        format!("{place} passes `{expr}`, {why}")
    }
}

fn route(place: &str, specifier: &str, form: &str) -> String {
    if specifier.is_empty() {
        format!("callers are unknowable because {place} uses {form}")
    } else {
        format!("callers are unknowable because {place} imports `{specifier}` through {form}")
    }
}

pub(super) fn kind_of(
    kind: crate::codebase::ts_source::module_facts::arguments::RouteKind,
) -> CauseKind {
    use crate::codebase::ts_source::module_facts::arguments::RouteKind;
    match kind {
        RouteKind::Namespace => CauseKind::NamespaceImport,
        RouteKind::Default => CauseKind::DefaultImport,
        RouteKind::Dynamic => CauseKind::DynamicImport,
        RouteKind::Require => CauseKind::RequireImport,
        RouteKind::ViMock => CauseKind::ViMock,
        RouteKind::NonDirect => CauseKind::NonDirectReference,
    }
}

pub(super) fn line_at(facts: &dyn TsFactLookup, file: &Path, offset: u32) -> u32 {
    facts
        .get_ts_facts(file)
        .and_then(|facts| facts.source.as_deref())
        .map(|source| crate::codebase::ts_source::byte_offset_to_line(source, offset as usize))
        .unwrap_or(1)
}

pub(super) fn function_site(facts: &dyn TsFactLookup, source: &NodeId) -> (PathBuf, u32) {
    let NodeId::Symbol {
        file, callable_id, ..
    } = source
    else {
        return (PathBuf::new(), 1);
    };
    let path = file.to_path_buf();
    let line = line_at(facts, &path, callable_id.map(|id| id.0).unwrap_or(0));
    (path, line)
}

/// No direct caller. An anonymous or unnamed default export is not a named function.
pub(super) fn missing_caller(facts: &dyn TsFactLookup, source: &NodeId, line: u32) -> Cause {
    let (file, _) = function_site(facts, source);
    let kind = if unnamed(source) {
        CauseKind::NotNamedFunction
    } else {
        CauseKind::NoCaller
    };
    Cause::new(kind, file, line, "")
}

fn unnamed(source: &NodeId) -> bool {
    let NodeId::Symbol { symbol, .. } = source else {
        return true;
    };
    let name = symbol.rsplit(['/', '.']).next().unwrap_or(symbol);
    name.starts_with("<anonymous") || name == "default"
}
