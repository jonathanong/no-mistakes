use super::pins::Resolver;
use crate::codebase::postgres::idents::object_name_key;
use crate::codebase::postgres::statements::{SqlBoundPin, SqlPinSource};
use sqlparser::ast::{JoinConstraint, JoinOperator, ObjectName};

/// `left JOIN right USING (columns)` of two single items, and the items it restricts.
pub(super) struct Using<'a> {
    restricted: Vec<usize>,
    left: usize,
    right: usize,
    columns: &'a [ObjectName],
}

impl<'a> Using<'a> {
    /// The `USING (…)` of an inner or outer join (a FULL join pins nothing). `USING` names a
    /// column of the left side, which is unambiguous only when the left side is one item.
    pub(super) fn of(
        operator: &'a JoinOperator,
        restricted: Vec<usize>,
        left: usize,
        right: usize,
    ) -> Option<Self> {
        let columns = match operator {
            JoinOperator::Join(JoinConstraint::Using(columns))
            | JoinOperator::Inner(JoinConstraint::Using(columns))
            | JoinOperator::Left(JoinConstraint::Using(columns))
            | JoinOperator::LeftOuter(JoinConstraint::Using(columns))
            | JoinOperator::Right(JoinConstraint::Using(columns))
            | JoinOperator::RightOuter(JoinConstraint::Using(columns)) => columns,
            _ => return None,
        };
        Some(Self {
            restricted,
            left,
            right,
            columns,
        })
    }

    /// Each restricted base table is pinned on the columns to the other side.
    pub(super) fn pins(
        &self,
        resolver: &Resolver,
        _positions: super::super::value::PlaceholderPositions<'_>,
        out: &mut Vec<(usize, SqlBoundPin)>,
    ) {
        for (pinned, other) in [(self.left, self.right), (self.right, self.left)] {
            if !self.restricted.contains(&pinned) || !resolver.is_table(pinned) {
                continue;
            }
            for column in self.columns {
                out.push((
                    pinned,
                    SqlBoundPin {
                        column: object_name_key(column),
                        source: SqlPinSource::Items(vec![other]),
                        null_safe: false,
                        reads: Vec::new(),
                    },
                ));
            }
        }
    }
}
