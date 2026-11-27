use crate::idents::ident_key;
use sqlparser::ast::{JoinConstraint, JoinOperator, ObjectName};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct UsingColumns {
    single_binding: Option<usize>,
    base_relation: Option<String>,
    merged: BTreeMap<String, usize>,
}

impl UsingColumns {
    pub(super) fn for_binding(binding: Option<usize>, base_relation: Option<String>) -> Self {
        Self {
            single_binding: binding,
            base_relation,
            ..Self::default()
        }
    }

    pub(super) fn resolve(&self, column: &str) -> Option<usize> {
        self.merged.get(column).copied().or(self.single_binding)
    }

    pub(super) fn advance(
        &mut self,
        op: &JoinOperator,
        constraint: Option<&JoinConstraint>,
        right_relation: Option<&str>,
    ) {
        match constraint {
            Some(JoinConstraint::Using(columns)) => {
                if right_relation.is_some()
                    && self.base_relation.as_deref() == right_relation
                    && preserves_left(op)
                {
                    for column in columns.iter().filter_map(column_name) {
                        if let Some(binding) = self.resolve(&column) {
                            self.merged.insert(column, binding);
                        }
                    }
                } else {
                    for column in columns.iter().filter_map(column_name) {
                        self.merged.remove(&column);
                    }
                }
            }
            Some(JoinConstraint::Natural) => self.merged.clear(),
            _ => {}
        }
        self.single_binding = None;
    }
}

fn preserves_left(op: &JoinOperator) -> bool {
    matches!(
        op,
        JoinOperator::Join(_)
            | JoinOperator::Inner(_)
            | JoinOperator::Left(_)
            | JoinOperator::LeftOuter(_)
    )
}

pub(super) fn column_name(column: &ObjectName) -> Option<String> {
    let [part] = column.0.as_slice() else {
        return None;
    };
    part.as_ident().map(ident_key)
}
