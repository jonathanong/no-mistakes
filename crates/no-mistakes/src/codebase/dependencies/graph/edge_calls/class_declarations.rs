/// A class with a statically named `extends` base. The base itself is the
/// [`EdgeKind::Extends`] edge leaving [`ClassDeclaration::node`]; this record
/// carries what an edge cannot: where the class is declared, whether it is
/// exported, ambient or a namespace member, and a base that names a global
/// rather than a repository class.
/// Empty unless the graph was built with [`GraphBuildPlan::extends`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassDeclaration {
    pub file: std::path::PathBuf,
    /// Display scope of the class within `file`.
    pub scope: String,
    pub callable_id: crate::codebase::dependencies::extract::CallableId,
    /// One-based declaration line; zero when the source text was not
    /// available at extraction.
    pub line: u32,
    /// Whether the module exports the class under any name.
    pub exported: bool,
    /// Declared with `declare`, or inside `declare namespace`, `declare module
    /// 'x'` or `declare global`: it describes code outside the analyzed
    /// source, so it is not a class of this project.
    pub ambient: bool,
    /// Dotted path of the namespace that declares the class, such as
    /// `Errors.Inner`. `None` for a class outside a tracked namespace.
    pub namespace: Option<String>,
    /// A use of the class's namespace is not a reference the graph resolves
    /// to a class: an alias, an argument, a computed access, `export
    /// default`, `export =`, a module read whole or at runtime, a merged
    /// declaration. Whether the class is constructed is then unknowable.
    pub namespace_escaped: bool,
    /// The global the base resolved to, such as `Error`. A global has no graph
    /// node, so it cannot be an `Extends` edge target.
    pub global_base: Option<String>,
}

impl ClassDeclaration {
    /// The node `Extends` edges leave from.
    pub fn node(&self) -> NodeId {
        NodeId::callable(&self.file, self.scope.as_str(), self.callable_id)
    }
}

impl DepGraph {
    pub fn class_declarations(&self) -> &[ClassDeclaration] {
        &self.class_declarations
    }
}
