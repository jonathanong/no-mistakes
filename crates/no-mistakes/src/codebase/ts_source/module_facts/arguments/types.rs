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
    /// A directory created and owned by the test through `mkdtemp`.
    Temp,
    /// The process-wide temporary directory, which the test does not own.
    SharedTemp,
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
    /// The `process.env` variable the leading component reads, if any.
    pub(crate) env: Option<String>,
    /// Index of the enclosing function's parameter the leading component reads.
    pub(crate) param: Option<usize>,
}

/// Where a `process.env` write sits relative to the code it must dominate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WriteScope {
    /// An unconditional top-level statement of the module.
    Program,
    /// An unconditional statement of a function body spanning these offsets.
    Function(u32, u32),
    /// An unconditional statement of a `beforeAll`/`beforeEach` callback whose
    /// registering scope spans these offsets.
    Hook(u32, u32),
    /// Nested in a branch, loop, or handler, so it may not have run.
    Conditional,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnvWrite {
    pub(crate) name: String,
    pub(crate) end: u32,
    pub(crate) origin: PathOrigin,
    pub(crate) scope: WriteScope,
}

/// Facts that let a rule trace a path through callers, env vars, and mocks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct PathProvenance {
    /// Call offset to `(argument index, origin)` for path-like arguments of calls
    /// through an imported binding.
    pub(crate) arg_paths: FxHashMap<u32, Vec<(usize, PathOrigin)>>,
    /// Calls that forward their enclosing mock implementation's rest arguments unchanged.
    pub(crate) pass_through: crate::fx::FxHashMap<u32, String>,
    pub(crate) env_writes: Vec<EnvWrite>,
    /// A `process.env` mutation whose variable cannot be named.
    pub(crate) env_unknown_write: bool,
    /// Function spans of `it`/`test` callbacks.
    pub(crate) test_callbacks: Vec<(u32, u32)>,
    /// Names of functions referenced other than as the callee of a direct call.
    pub(crate) value_refs: crate::fx::FxHashSet<String>,
}
