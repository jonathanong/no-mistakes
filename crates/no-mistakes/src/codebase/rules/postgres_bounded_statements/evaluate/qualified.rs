use crate::codebase::postgres::statements::SqlQualifiedRead;
use crate::codebase::postgres::SchemaCatalog;

/// Whether a qualified read refers to a relation in its local scopes.
///
/// A matching unaliased table proves locality only when both names resolve to the same catalog
/// relation. Unknown relation kinds and catalog entries remain conservative because they may
/// shadow the outer relation.
pub(super) fn reads_outer(reads: &[SqlQualifiedRead], catalog: &SchemaCatalog) -> bool {
    reads.iter().any(|read| {
        if catalog.relation(&read.qualifier).is_none() {
            return true;
        }
        for scope in &read.scopes {
            if scope.unknown {
                return true;
            }
            let mut resolved_scope = false;
            for name in &scope.tables {
                if catalog.relation(name).is_none() {
                    return true;
                }
                if catalog.same_relation(&read.qualifier, name) {
                    resolved_scope = true;
                    break;
                }
            }
            if resolved_scope {
                return false;
            }
        }
        true
    })
}
