use super::*;

#[derive(Clone, PartialEq, Eq)]
pub(super) enum FlowValue {
    Uncallable,
    Choice(Vec<FlowValue>),
    TypedObject {
        executor: Box<FlowValue>,
        properties: FxHashMap<String, FlowValue>,
    },
    Provenance {
        executor: Box<FlowValue>,
        typed: Vec<(String, String)>,
        members: Vec<(String, String, String)>,
        receiver_factories: Vec<(String, String)>,
    },
    Promise(Box<FlowValue>),
    Bound {
        target: Box<FlowValue>,
        arguments: Vec<BoundArguments>,
    },
    Target(ResolvedCallTarget),
    Returned {
        factory: Vec<(String, String)>,
        executor: Box<FlowValue>,
        receiver: Option<ResolvedCallTarget>,
    },
    Callable {
        file: std::path::PathBuf,
        scope: String,
        id: CallableId,
    },
    Object {
        null_prototype: bool,
        file: std::path::PathBuf,
        fields: std::sync::Arc<FxHashMap<String, ValueExpression>>,
    },
    Array {
        file: std::path::PathBuf,
        fields: std::sync::Arc<FxHashMap<String, ValueExpression>>,
    },
    Undefined,
    Null,
    Unknown,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct BoundArguments {
    pub(super) file: std::path::PathBuf,
    pub(super) arguments: std::sync::Arc<Vec<ValueExpression>>,
    pub(super) spread_at: Option<usize>,
}

impl FlowValue {
    // Metadata must not change ordinary value-flow operations.
    pub(super) fn map_executor(self, operation: impl FnOnce(Self) -> Self) -> Self {
        match self {
            Self::Provenance {
                executor,
                typed,
                members,
                receiver_factories,
            } => Self::Provenance {
                executor: Box::new(executor.map_executor(operation)),
                typed,
                members,
                receiver_factories,
            },
            Self::TypedObject { executor, properties } => Self::TypedObject {
                executor: Box::new(executor.map_executor(operation)), properties,
            },
            value => operation(value),
        }
    }
    pub(super) fn executor(&self) -> &Self {
        match self {
            Self::Provenance { executor, .. } | Self::TypedObject { executor, .. } => executor.executor(),
            value => value,
        }
    }
    pub(super) fn is_undefined(&self) -> bool {
        matches!(self.executor(), Self::Undefined)
    }
    pub(super) fn awaited(self) -> Self {
        self.map_executor(|value| match value {
            Self::Promise(inner) => inner.awaited(),
            value => value,
        })
    }
}
