use super::{Frame, Qualified, Scope, SqlBareRead};
use std::collections::BTreeMap;

pub(super) struct JoinScope {
    outside: Scope,
    qualifiers: usize,
    escaping_qualifiers: usize,
    bare: BTreeMap<String, usize>,
}

impl JoinScope {
    pub(super) fn begin(frame: &mut Frame) -> Self {
        Self {
            outside: frame.scope.begin_join(),
            qualifiers: frame.qualifiers.len(),
            escaping_qualifiers: frame.escaping_qualifiers.len(),
            bare: frame.bare.clone(),
        }
    }

    pub(super) fn finish(self, frame: &mut Frame) {
        // ON sees the completed children, but never the alias assigned after this join.
        let candidates = frame.scope.qualified_candidates();
        let internal = frame.qualifiers.split_off(self.qualifiers);
        frame.join_qualifiers.extend(
            internal
                .into_iter()
                .filter(|read| !frame.scope.relations.contains(&read.key))
                .map(|mut read: Qualified| {
                    read.scopes.push(candidates.clone());
                    read
                }),
        );
        // LATERAL already used its preceding-source snapshot. Later children cannot shadow it.
        frame.join_qualifiers.extend(
            frame
                .escaping_qualifiers
                .split_off(self.escaping_qualifiers),
        );
        let mut reads = Vec::new();
        frame.bare.retain(|name, count| {
            let before = self.bare.get(name).copied().unwrap_or(0);
            if *count > before && !frame.scope.whole_rows.contains(name) {
                reads.push(SqlBareRead {
                    column: name.clone(),
                    tables: Vec::new(),
                });
            }
            *count = before;
            before > 0
        });
        frame.scope.resolve_reads(&mut reads);
        frame.join_reads.extend(reads);
        frame.scope.finish_join(self.outside);
    }
}
