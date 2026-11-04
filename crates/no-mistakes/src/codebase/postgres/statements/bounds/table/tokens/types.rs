use sqlparser::ast::Ident;

/// Source spellings for TABLE query arms. sqlparser's `Table` AST stores only word values,
/// so a quoted mixed-case name needs the already prepared token stream to retain identity.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(in crate::codebase::postgres) struct TableTokenCursor {
    pub(super) names: Vec<SourceTable>,
    pub(super) operators: Vec<SourceOperator>,
    pub(super) froms: Vec<SourceOperator>,
    pub(super) depths: Vec<SourceDepth>,
    pub(super) next: usize,
    pub(super) last_at: Option<(usize, usize)>,
    pub(super) last_depth: usize,
    pub(super) last_operator: Option<(usize, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SourceOperator {
    pub(super) at: (usize, usize),
    pub(super) depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SourceDepth {
    pub(super) at: (usize, usize),
    pub(super) depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::codebase::postgres) struct SourceTable {
    pub(super) schema: Option<String>,
    pub(super) table: String,
    pub(super) parts: Vec<Ident>,
    pub(in crate::codebase::postgres) name: String,
    pub(in crate::codebase::postgres) key: String,
    pub(in crate::codebase::postgres) at: (usize, usize),
    pub(super) depth: usize,
}
