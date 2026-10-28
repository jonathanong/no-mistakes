use crate::fx::FxHashMap;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ArgumentValue {
    Scalar(Value),
    Array(Vec<ArgumentValue>),
    Object(FxHashMap<String, ArgumentValue>),
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ArgumentValues {
    pub(crate) values: Vec<ArgumentValue>,
    pub(crate) spread_from: Option<usize>,
    /// Where the first argument's path comes from, for calls through an
    /// imported binding. `None` for every other call.
    pub(crate) path: Option<PathOrigin>,
}

/// The base a path expression is resolved against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PathAnchor {
    /// The directory or URL of the current module.
    Module,
    /// `process.cwd()`, which is the live repository for a test run.
    Cwd,
    /// `os.tmpdir()` or the result of `mkdtemp`.
    Temp,
    /// A relative literal, resolved against the process working directory.
    Relative,
    Absolute,
    /// A parameter, mutable binding, or any expression this pass cannot follow.
    Unknown,
}

/// Static provenance of a path-valued expression; it never evaluates code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PathOrigin {
    pub(crate) anchor: PathAnchor,
    /// Literal segments in source order, anchor excluded.
    pub(crate) segments: Vec<String>,
    /// True when a component after the anchor is not a literal.
    pub(crate) dynamic: bool,
}
