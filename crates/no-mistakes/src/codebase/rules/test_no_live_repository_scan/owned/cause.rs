//! Why a parameter root stayed unproven. Acceptance stays in the walk.
#[path = "cause_text.rs"]
mod cause_text;

use crate::codebase::dependencies::graph::{NodeId, TsFactLookup};
use std::path::{Path, PathBuf};

pub(in super::super) enum Ownership {
    Yes,
    No,
    Parameter(Cause),
}

#[derive(Clone)]
pub(in super::super) struct Cause {
    pub(super) kind: CauseKind,
    pub(super) file: PathBuf,
    pub(super) line: u32,
    pub(super) expression: String,
    pub(super) property: Option<String>,
    /// Blockers after the one named in the message.
    pub(super) extra: u32,
    /// Test-root path that reaches an unread module, when one is known.
    pub(super) reached: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    /// Vitest config could not be read, or no project matches the selected tests.
    UnreadableConfig,
    /// The function site is a self-recursive inner helper, not an unread module.
    RecursiveHelper,
    /// No proven caller, and no unread module, at the function site.
    CallersNotProven,
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
            property: None,
            extra: 0,
            reached: String::new(),
        }
    }

    pub(super) fn with_property(mut self, property: Option<String>) -> Self {
        self.property = property;
        self
    }

    pub(super) fn with_extra(mut self, extra: u32) -> Self {
        self.extra = extra;
        self
    }

    pub(super) fn with_reached(mut self, reached: impl Into<String>) -> Self {
        self.reached = reached.into();
        self
    }

    pub(in super::super) fn detail(&self, root: &Path) -> String {
        cause_text::detail(self, root)
    }

    pub(in super::super) fn fix(&self) -> &'static str {
        cause_text::fix(self.kind)
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
