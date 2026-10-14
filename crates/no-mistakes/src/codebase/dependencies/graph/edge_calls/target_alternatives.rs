/// A leaf callable identity. Alternative sets cannot contain nested sets.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum KnownCallTarget {
    Global { name: String },
    ModuleExport {
        specifier: String,
        export_path: String,
        repository_target: Option<(std::path::PathBuf, String)>,
        callable_id: Option<crate::codebase::dependencies::extract::CallableId>,
    },
    RepositoryFunction { file: std::path::PathBuf, scope: String },
}

/// A callable candidate and its existing canonical repository node, if any.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResolvedCallCandidate {
    pub target: KnownCallTarget,
    pub node: Option<NodeId>,
}

/// Shared, deterministic possibilities for one physical call occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCallAlternatives {
    known: Vec<ResolvedCallCandidate>,
    /// Other callable values could not be resolved; known candidates still count.
    pub has_unknown: bool,
    nodes: Vec<NodeId>,
}

/// Borrowed target identity; ordinary iteration does not clone names or paths.
#[derive(Debug, Clone, Copy)]
pub enum KnownCallTargetRef<'a> {
    Global { name: &'a str },
    ModuleExport {
        specifier: &'a str,
        export_path: &'a str,
        repository_target: Option<(&'a std::path::Path, &'a str)>,
        callable_id: Option<crate::codebase::dependencies::extract::CallableId>,
    },
    RepositoryFunction { file: &'a std::path::Path, scope: &'a str },
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedCallCandidateRef<'a> {
    pub target: KnownCallTargetRef<'a>,
    pub node: Option<&'a NodeId>,
}

impl KnownCallTarget {
    pub fn as_ref(&self) -> KnownCallTargetRef<'_> {
        match self {
            Self::Global { name } => KnownCallTargetRef::Global { name },
            Self::ModuleExport { specifier, export_path, repository_target, callable_id } => {
                KnownCallTargetRef::ModuleExport {
                    specifier, export_path, callable_id: *callable_id,
                    repository_target: repository_target.as_ref().map(|(file, scope)| (file.as_path(), scope.as_str())),
                }
            }
            Self::RepositoryFunction { file, scope } => KnownCallTargetRef::RepositoryFunction { file, scope },
        }
    }
}

impl ResolvedCallTarget {
    pub fn as_known(&self) -> Option<KnownCallTargetRef<'_>> {
        Some(match self {
            Self::Global { name } => KnownCallTargetRef::Global { name },
            Self::ModuleExport { specifier, export_path, repository_target, callable_id } => {
                KnownCallTargetRef::ModuleExport {
                    specifier, export_path, callable_id: *callable_id,
                    repository_target: repository_target.as_ref().map(|(file, scope)| (file.as_path(), scope.as_str())),
                }
            }
            Self::RepositoryFunction { file, scope } => KnownCallTargetRef::RepositoryFunction { file, scope },
            Self::Unknown | Self::Alternatives(_) => return None,
        })
    }
}

/// An allocation-free view of the known targets of one physical occurrence.
pub struct ResolvedCallTargets<'a> {
    single: Option<ResolvedCallCandidateRef<'a>>,
    alternatives: std::slice::Iter<'a, ResolvedCallCandidate>,
}

impl<'a> Iterator for ResolvedCallTargets<'a> {
    type Item = ResolvedCallCandidateRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.single.take().or_else(|| self.alternatives.next().map(|candidate| {
            ResolvedCallCandidateRef { target: candidate.target.as_ref(), node: candidate.node.as_ref() }
        }))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = usize::from(self.single.is_some()) + self.alternatives.len();
        (length, Some(length))
    }
}
impl ExactSizeIterator for ResolvedCallTargets<'_> {}

impl ResolvedCallSite {
    pub fn targets(&self) -> ResolvedCallTargets<'_> {
        if let ResolvedCallTarget::Alternatives(alternatives) = &self.target {
            ResolvedCallTargets { single: None, alternatives: alternatives.known.iter() }
        } else {
            ResolvedCallTargets {
                single: self.target.as_known().map(|target| ResolvedCallCandidateRef { target, node: self.target_node.as_ref() }),
                alternatives: [].iter(),
            }
        }
    }
    pub fn has_unknown_target(&self) -> bool {
        match &self.target {
            ResolvedCallTarget::Unknown => true,
            ResolvedCallTarget::Alternatives(alternatives) => alternatives.has_unknown,
            _ => false,
        }
    }
}

include!("target_nodes.rs");
