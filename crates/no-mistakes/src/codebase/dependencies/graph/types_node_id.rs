pub(crate) fn intern_node_path(path: impl AsRef<Path>) -> Arc<Path> {
    Arc::from(crate::codebase::ts_resolver::normalize_path(path.as_ref()))
}

pub(crate) fn intern_node_str(value: impl Into<Arc<str>>) -> Arc<str> {
    value.into()
}

impl PartialEq for NodeId {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::File(a), Self::File(b)) => a == b,
            (
                Self::Symbol {
                    file: fa,
                    symbol: sa,
                    callable_id: ia,
                },
                Self::Symbol {
                    file: fb,
                    symbol: sb,
                    callable_id: ib,
                },
            ) => fa == fb && sa == sb && ia == ib,
            (Self::Module(a), Self::Module(b)) => a == b,
            (
                Self::QueueJob {
                    queue_file: fa,
                    job: ja,
                },
                Self::QueueJob {
                    queue_file: fb,
                    job: jb,
                },
            ) => fa == fb && ja == jb,
            (
                Self::WorkflowJob {
                    workflow_file: fa,
                    job: ja,
                },
                Self::WorkflowJob {
                    workflow_file: fb,
                    job: jb,
                },
            ) => fa == fb && ja == jb,
            (
                Self::WorkflowStep {
                    workflow_file: fa,
                    job: ja,
                    step: sa,
                },
                Self::WorkflowStep {
                    workflow_file: fb,
                    job: jb,
                    step: sb,
                },
            ) => fa == fb && ja == jb && sa == sb,
            (
                Self::TrpcProcedure {
                    router_file: fa,
                    procedure: pa,
                },
                Self::TrpcProcedure {
                    router_file: fb,
                    procedure: pb,
                },
            ) => fa == fb && pa == pb,
            _ => false,
        }
    }
}

impl Eq for NodeId {}

impl Hash for NodeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::File(path) => path.hash(state),
            Self::Symbol {
                file,
                symbol,
                callable_id,
            } => {
                file.hash(state);
                symbol.hash(state);
                callable_id.hash(state);
            }
            Self::Module(specifier) => specifier.hash(state),
            Self::QueueJob { queue_file, job } => {
                queue_file.hash(state);
                job.hash(state);
            }
            Self::WorkflowJob { workflow_file, job } => {
                workflow_file.hash(state);
                job.hash(state);
            }
            Self::WorkflowStep {
                workflow_file,
                job,
                step,
            } => {
                workflow_file.hash(state);
                job.hash(state);
                step.hash(state);
            }
            Self::TrpcProcedure {
                router_file,
                procedure,
            } => {
                router_file.hash(state);
                procedure.hash(state);
            }
        }
    }
}

include!("types_node_id_constructors.rs");

include!("types_node_id_trpc.rs");
