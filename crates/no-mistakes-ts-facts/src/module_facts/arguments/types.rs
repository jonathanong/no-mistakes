mod array_guard;
mod path_default;
pub use super::path_value::{MemberEffect, PathValueKind};
use super::ConditionalPlace;
use crate::fx::FxHashMap;
pub use array_guard::ArrayGuard;
pub use path_default::PathDefault;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentValue {
    Scalar(Value),
    Array(Vec<ArgumentValue>),
    Object(FxHashMap<String, ArgumentValue>),
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArgumentValues {
    pub values: Vec<ArgumentValue>,
    pub spread_from: Option<usize>,
    /// Where the first argument's path comes from, for calls through an
    /// imported binding. `None` for every other call.
    pub path: Option<PathOrigin>,
}

/// The base a path expression is resolved against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathAnchor {
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
#[derive(Debug, Clone)]
pub struct PathOrigin {
    pub value_kind: PathValueKind,
    pub require_scalar: bool,
    pub relative: bool,
    pub nullish: bool,
    pub anchor: PathAnchor,
    /// Literal segments in source order, anchor excluded.
    pub segments: Vec<String>,
    /// True when a component after the anchor is not a literal.
    pub dynamic: bool,
    /// The `process.env` variable the leading component reads, if any.
    pub env: Option<String>,
    pub env_diagnostic: Option<Box<super::EnvDiagnostic>>,
    pub captured_getter: Option<u32>,
    /// Offset of a named imported zero-argument getter call.
    pub getter: Option<u32>,
    pub root_tails: Vec<PathOrigin>,
    pub captured_env: Option<Box<super::CapturedEnv>>,
    /// Index of the enclosing function's parameter the leading component reads.
    pub param: Option<usize>,
    pub param_member: Option<String>,
    pub param_owner: Option<u32>,
    /// 1-based line of `param_owner`, recorded while the source is in hand.
    pub owner_line: Option<u32>,
    /// 1-based line of a write to a simple parameter. Set only while `param`
    /// stays unset, so caller-proof acceptance is unchanged.
    pub reassigned_at: Option<u32>,
    pub alternatives: Vec<PathOrigin>,
    pub param_default: Option<PathDefault>,
}

/// Where a `process.env` write sits relative to the code it must dominate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteScope {
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
    Conditional(ConditionalPlace),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvWrite {
    pub line: u32,
    pub cleared: bool,
    pub name: String,
    pub end: u32,
    pub origin: PathOrigin,
    pub scope: WriteScope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentText {
    pub index: usize,
    pub property: Option<String>,
    pub line: u32,
    pub text: String,
}

/// Facts that let a rule trace a path through callers, env vars, and mocks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PathProvenance {
    pub arg_counts: FxHashMap<u32, Option<usize>>,
    pub arg_member_effects: FxHashMap<u32, Vec<(usize, MemberEffect)>>,
    pub arg_closed_objects: FxHashMap<u32, Vec<usize>>,
    pub arg_closed_members: FxHashMap<u32, Vec<(usize, String)>>,
    pub call_guards: FxHashMap<u32, Vec<ArrayGuard>>,
    pub arg_member_paths: FxHashMap<u32, Vec<(usize, String, PathOrigin)>>,
    pub arg_paths: FxHashMap<u32, Vec<(usize, PathOrigin)>>,
    /// Calls that forward their enclosing mock implementation's rest arguments unchanged.
    pub pass_through: crate::fx::FxHashMap<u32, String>,
    pub env_writes: Vec<EnvWrite>,
    pub env_resets: Vec<WriteScope>,
    /// Env-root returns keyed by callable start offset.
    pub env_getters: FxHashMap<u32, PathOrigin>,
    pub parameter_defaults: FxHashMap<u32, FxHashMap<usize, PathOrigin>>,
    /// A `process.env` mutation whose variable cannot be named.
    pub env_unknown_write: bool,
    pub env_opaque_write: bool,
    pub test_env_calls: FxHashMap<u32, u32>,
    pub immediate_env_reads: crate::fx::FxHashSet<u32>,
    /// A computed object key whose text is not a literal other than `unstubEnvs`.
    pub opaque_config_key: bool,
    /// Function spans of `it`/`test` callbacks.
    pub test_callbacks: Vec<(u32, u32)>,
    pub cleanup_callbacks: Vec<(u32, u32)>,
    pub lifecycle_overlaps: Vec<(u32, u32)>,
    /// Names of functions referenced other than as the callee of a direct call.
    pub value_refs: crate::fx::FxHashSet<String>,
    /// Specifiers reached other than by named static imports used only as calls.
    pub unsafe_modules: crate::fx::FxHashSet<String>,
    /// 1-based line of each static import, saved while the source text exists.
    pub import_lines: FxHashMap<String, u32>,
    /// A `require`/`import()`/`vi.mock` whose module cannot be read.
    pub modules_unknown: bool,
    /// 1-based line of the first load that set `modules_unknown`.
    pub unknown_load: Option<u32>,
    pub resolved_import_spans: Vec<(u32, u32)>,
    /// Diagnostic text and locations for positional arguments and direct literal properties.
    pub arg_text: FxHashMap<u32, Vec<ArgumentText>>,
    /// Origins omitted from `arg_paths`. The caller proof does not read this.
    pub arg_unproven: FxHashMap<u32, Vec<(usize, PathOrigin)>>,
    /// Imports and references that can make a callee's callers unknowable.
    pub route_sites: Vec<RouteSite>,
    pub registry_dispatches: Vec<(u32, String, Option<String>, String)>,
    pub absent_or_non_finite_args: FxHashMap<u32, Vec<(usize, Option<String>)>>,
    pub instance_properties: FxHashMap<u32, super::instance_properties::InstancePropertyEvidence>,
    pub call_derived_constructs: crate::fx::FxHashSet<u32>,
    pub dynamic_scope: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteSite {
    pub start: u32,
    pub line: u32,
    pub kind: RouteKind,
    pub detail: String,
}
pub fn line_in(source: &str, offset: u32) -> u32 {
    crate::syntax_helpers::byte_offset_to_line(source, offset as usize)
}
impl RouteSite {
    pub fn at(source: &str, start: u32, kind: RouteKind, detail: impl Into<String>) -> Self {
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
pub enum RouteKind {
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
