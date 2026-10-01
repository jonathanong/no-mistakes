/// A class declared in a TypeScript namespace body, named by its dotted path
/// from the file's top-level namespace (`Errors.Inner.DeepError`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceMember {
    pub path: String,
    pub id: CallableId,
    /// Reachable from another module: the class and every namespace between it
    /// and the root are `export`ed, and the file exports the root.
    pub exported: bool,
}

/// A top-level (non-ambient) namespace declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceRoot {
    pub name: String,
    /// The names the file exports the namespace as: `export namespace X`, an
    /// `export { X as Y }` clause, or both.
    pub exports: Vec<String>,
    /// A class, function, variable, enum or import shares the name, so a member
    /// access may mean either declaration.
    pub merged: bool,
}

/// A construction written inside a namespace body, so a bare `new Local()` can
/// be resolved against the members of the namespaces around it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceSite {
    pub caller_id: Option<CallableId>,
    pub offset: u32,
    /// Dotted path of the innermost enclosing namespace.
    pub namespace: String,
}

/// Namespace declarations and the uses of their names in one file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NamespaceFacts {
    pub members: Vec<NamespaceMember>,
    pub roots: Vec<NamespaceRoot>,
    /// Every non-ambient namespace path, nested and dotted ones included.
    pub declared: Vec<String>,
    /// Names that appear as a value other than the head of a call, `new`,
    /// `extends`, `instanceof` or re-export: an alias, an argument, a computed
    /// access, `export default`. Only names of declared namespaces and
    /// imports are recorded.
    pub value_uses: Vec<String>,
    pub sites: Vec<NamespaceSite>,
    /// Specifiers of `import x = require("...")`, whose module is used whole.
    pub opaque_specifiers: Vec<String>,
    /// Classes in an ambient declaration or in a module block that is not a
    /// tracked namespace: `declare class`, `declare module`, `declare global`.
    pub unreported_class_ids: Vec<CallableId>,
}

/// Walk state for [`NamespaceFacts`]. The pre-scan fills `facts` before the
/// walk, which adds sites and uses.
#[derive(Default)]
struct NamespaceState {
    facts: NamespaceFacts,
    member_ids: FxHashSet<CallableId>,
    /// Names whose value uses matter: namespace segments and import locals.
    names: FxHashSet<String>,
    /// Dotted path of each namespace the walk is inside.
    stack: Vec<String>,
    /// Source offsets of identifiers that head a position the graph resolves.
    benign_heads: FxHashSet<u32>,
    /// Declared namespace paths and import names read as a value.
    value_uses: FxHashSet<String>,
    /// Nesting depth of erased type names (`typeof X`, `implements X.I`).
    type_depth: u32,
}
