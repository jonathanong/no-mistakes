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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PathOrigin {
    pub(crate) anchor: PathAnchor,
    /// Literal segments in source order, anchor excluded.
    pub(crate) segments: Vec<String>,
    /// True when a component after the anchor is not a literal.
    pub(crate) dynamic: bool,
    /// The `process.env` variable the leading component reads, if any.
    pub(crate) env: Option<String>,
    /// Offset of a named imported zero-argument getter call.
    pub(crate) getter: Option<u32>,
    pub(crate) root_tails: Vec<PathOrigin>,
    /// Index of the enclosing function's parameter the leading component reads.
    pub(crate) param: Option<usize>,
    pub(crate) param_member: Option<String>,
    pub(crate) param_owner: Option<u32>,
    /// 1-based line of `param_owner`, recorded while the source is in hand.
    pub(crate) owner_line: Option<u32>,
    /// 1-based line of a write to a simple parameter. Set only while `param`
    /// stays unset, so caller-proof acceptance is unchanged.
    pub(crate) reassigned_at: Option<u32>,
    pub(crate) alternatives: Vec<PathOrigin>,
    pub(crate) param_default: Option<Box<PathOrigin>>,
}

/// Where a `process.env` write sits relative to the code it must dominate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WriteScope {
    /// An unconditional top-level statement of the module.
    Program,
    /// An unconditional statement of a function body spanning these offsets.
    Function(u32, u32),
    /// An unconditional statement of a `beforeAll` callback whose
    /// registering scope spans these offsets.
    Hook(u32, u32),
    BeforeEach(u32, u32),
    AfterEach(u32, u32),
    AfterAll(u32, u32),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArgumentText {
    pub(crate) index: usize,
    pub(crate) property: Option<String>,
    pub(crate) line: u32,
    pub(crate) text: String,
}

/// Facts that let a rule trace a path through callers, env vars, and mocks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct PathProvenance {
    pub(crate) arg_counts: FxHashMap<u32, Option<usize>>,
    pub(crate) arg_closed_objects: FxHashMap<u32, Vec<usize>>,
    pub(crate) arg_member_paths: FxHashMap<u32, Vec<(usize, String, PathOrigin)>>,
    pub(crate) arg_paths: FxHashMap<u32, Vec<(usize, PathOrigin)>>,
    /// Calls that forward their enclosing mock implementation's rest arguments unchanged.
    pub(crate) pass_through: crate::fx::FxHashMap<u32, String>,
    pub(crate) env_writes: Vec<EnvWrite>,
    pub(crate) env_resets: Vec<WriteScope>,
    /// Env-root returns keyed by callable start offset.
    pub(crate) env_getters: FxHashMap<u32, PathOrigin>,
    pub(crate) parameter_defaults: FxHashMap<u32, FxHashMap<usize, PathOrigin>>,
    /// A `process.env` mutation whose variable cannot be named.
    pub(crate) env_unknown_write: bool,
    pub(crate) env_opaque_write: bool,
    pub(crate) test_env_calls: FxHashMap<u32, u32>,
    pub(crate) immediate_env_reads: crate::fx::FxHashSet<u32>,
    /// A computed object key whose text is not a literal other than `unstubEnvs`.
    pub(crate) opaque_config_key: bool,
    /// Function spans of `it`/`test` callbacks.
    pub(crate) test_callbacks: Vec<(u32, u32)>,
    pub(crate) cleanup_callbacks: Vec<(u32, u32)>,
    pub(crate) lifecycle_overlaps: Vec<(u32, u32)>,
    /// Names of functions referenced other than as the callee of a direct call.
    pub(crate) value_refs: crate::fx::FxHashSet<String>,
    /// Specifiers reached other than by named static imports used only as calls.
    pub(crate) unsafe_modules: crate::fx::FxHashSet<String>,
    /// 1-based line of each static import, saved while the source text exists.
    pub(crate) import_lines: FxHashMap<String, u32>,
    /// A `require`/`import()`/`vi.mock` whose module cannot be read.
    pub(crate) modules_unknown: bool,
    /// 1-based line of the first load that set `modules_unknown`.
    pub(crate) unknown_load: Option<u32>,
    pub(crate) resolved_import_spans: Vec<(u32, u32)>,
    /// Diagnostic text and locations for positional arguments and direct literal properties.
    pub(crate) arg_text: FxHashMap<u32, Vec<ArgumentText>>,
    /// Origins omitted from `arg_paths`. The caller proof does not read this.
    pub(crate) arg_unproven: FxHashMap<u32, Vec<(usize, PathOrigin)>>,
    /// Imports and references that can make a callee's callers unknowable.
    pub(crate) route_sites: Vec<RouteSite>,
}

/// One import or reference that blocks a parameter-root proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RouteSite {
    pub(crate) start: u32,
    pub(crate) line: u32,
    pub(crate) kind: RouteKind,
    pub(crate) detail: String,
}

pub(crate) fn line_in(source: &str, offset: u32) -> u32 {
    crate::codebase::ts_source::byte_offset_to_line(source, offset as usize)
}

impl RouteSite {
    pub(crate) fn at(source: &str, start: u32, kind: RouteKind, detail: impl Into<String>) -> Self {
        Self {
            start,
            line: line_in(source, start),
            kind,
            detail: detail.into(),
        }
    }
}

/// Why a route into a parameter root cannot be enumerated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RouteKind {
    Namespace,
    Default,
    Dynamic,
    Require,
    ViMock,
    NonDirect,
}

impl PathProvenance {
    pub(super) fn poison_env(&mut self) {
        self.env_unknown_write = true;
        self.env_opaque_write = true;
    }
}
