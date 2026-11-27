use super::{Roots, SqlFunctionClause};
use sqlparser::ast::{JoinConstraint, JoinOperator, TableWithJoins};

impl Roots {
    pub(super) fn joins(&mut self, tables: &[TableWithJoins]) {
        for table in tables {
            for join in &table.joins {
                match &join.join_operator {
                    JoinOperator::Join(constraint)
                    | JoinOperator::Inner(constraint)
                    | JoinOperator::Left(constraint)
                    | JoinOperator::LeftOuter(constraint)
                    | JoinOperator::Right(constraint)
                    | JoinOperator::RightOuter(constraint)
                    | JoinOperator::FullOuter(constraint)
                    | JoinOperator::CrossJoin(constraint)
                    | JoinOperator::Semi(constraint)
                    | JoinOperator::LeftSemi(constraint)
                    | JoinOperator::RightSemi(constraint)
                    | JoinOperator::Anti(constraint)
                    | JoinOperator::LeftAnti(constraint)
                    | JoinOperator::RightAnti(constraint)
                    | JoinOperator::StraightJoin(constraint)
                    | JoinOperator::AsOf { constraint, .. } => {
                        if let JoinConstraint::On(expression) = constraint {
                            self.expr(expression, SqlFunctionClause::JoinOn);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
