use super::{Roots, SqlFunctionClause as Clause};
use sqlparser::ast::{
    AlterColumnOperation, AlterTableOperation, Assignment, ColumnDef, ColumnOption, FromTable,
    Insert, Merge, MergeAction, MergeInsertKind, MergeUpdateKind, OnConflictAction, OnInsert,
    OutputClause, Statement, UpdateTableFromKind,
};

impl Roots {
    pub(in crate::codebase::postgres::function_calls) fn statement(
        &mut self,
        statement: &Statement,
    ) {
        match statement {
            Statement::Insert(insert) => self.insert(insert),
            Statement::Update(update) => {
                self.assignments(&update.assignments);
                self.joins(std::slice::from_ref(&update.table));
                if let Some(
                    UpdateTableFromKind::BeforeSet(from) | UpdateTableFromKind::AfterSet(from),
                ) = &update.from
                {
                    self.joins(from);
                }
                if let Some(selection) = &update.selection {
                    self.expr(selection, Clause::Where);
                }
                if let Some(items) = &update.returning {
                    self.items(items, Clause::Returning);
                }
                self.order_exprs(&update.order_by);
            }
            Statement::Delete(delete) => {
                let (FromTable::WithFromKeyword(from) | FromTable::WithoutKeyword(from)) =
                    &delete.from;
                self.joins(from);
                if let Some(using) = &delete.using {
                    self.joins(using);
                }
                if let Some(selection) = &delete.selection {
                    self.expr(selection, Clause::Where);
                }
                if let Some(items) = &delete.returning {
                    self.items(items, Clause::Returning);
                }
                self.order_exprs(&delete.order_by);
            }
            Statement::Merge(merge) => self.merge(merge),
            Statement::CreateIndex(index) => {
                if let Some(predicate) = &index.predicate {
                    self.expr(predicate, Clause::Where);
                }
            }
            Statement::CreateTable(table) => self.defaults(&table.columns),
            Statement::AlterTable(table) => {
                for operation in &table.operations {
                    match operation {
                        AlterTableOperation::AlterColumn {
                            op: AlterColumnOperation::SetDefault { value },
                            ..
                        } => self.expr(value, Clause::Default),
                        AlterTableOperation::AddColumn { column_def, .. } => {
                            self.defaults(std::slice::from_ref(column_def))
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn insert(&mut self, insert: &Insert) {
        if let Some(on) = &insert.on {
            match on {
                OnInsert::DuplicateKeyUpdate(assignments) => self.assignments(assignments),
                OnInsert::OnConflict(conflict) => {
                    if let OnConflictAction::DoUpdate(update) = &conflict.action {
                        self.assignments(&update.assignments);
                        if let Some(selection) = &update.selection {
                            self.expr(selection, Clause::Where);
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some(items) = &insert.returning {
            self.items(items, Clause::Returning);
        }
    }

    fn merge(&mut self, merge: &Merge) {
        if let Some(OutputClause::Returning { select_items, .. }) = &merge.output {
            self.items(select_items, Clause::Returning);
        }
        self.expr(&merge.on, Clause::JoinOn);
        for clause in &merge.clauses {
            if let Some(predicate) = &clause.predicate {
                self.expr(predicate, Clause::Where);
            }
            match &clause.action {
                MergeAction::Update(update) => {
                    if let MergeUpdateKind::Set(assignments) = &update.kind {
                        self.assignments(assignments);
                    }
                    for expr in [&update.update_predicate, &update.delete_predicate]
                        .into_iter()
                        .flatten()
                    {
                        self.expr(expr, Clause::Where);
                    }
                }
                MergeAction::Insert(insert) => {
                    if let MergeInsertKind::Values(values) = &insert.kind {
                        for expr in values.rows.iter().flat_map(|row| row.iter()) {
                            self.expr(expr, Clause::Values);
                        }
                    }
                    if let Some(expr) = &insert.insert_predicate {
                        self.expr(expr, Clause::Where);
                    }
                }
                _ => {}
            }
        }
    }

    fn assignments(&mut self, assignments: &[Assignment]) {
        for assignment in assignments {
            self.expr(&assignment.value, Clause::Set);
        }
    }

    fn defaults(&mut self, columns: &[ColumnDef]) {
        for column in columns {
            for option in &column.options {
                if let ColumnOption::Default(expr) = &option.option {
                    self.expr(expr, Clause::Default);
                }
            }
        }
    }
}
