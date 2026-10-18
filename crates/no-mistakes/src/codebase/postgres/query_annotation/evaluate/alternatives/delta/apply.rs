use super::*;
impl Join {
    pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn apply<
        F: Fn(&str, &Path) -> Option<PathBuf>,
    >(
        &self,
        evaluator: &mut Evaluator<'_, F>,
    ) {
        let (mut bindings, mut dense, mut shape_cells) = (0_u64, 0_u64, 0_u64);
        let mut visited = 0_u64;
        for (cell, entry) in &self.entries {
            visited += 1;
            if matches!(cell, Cell::Length(id) if !self.shapes.contains(id)) {
                evaluator.apply_cell(cell, &entry.joined);
            }
        }
        for (cell, entry) in &self.entries {
            visited += 1;
            if matches!(
                cell,
                Cell::Length(_) | Cell::Fresh(..) | Cell::Disconnected(..)
            ) {
                continue;
            }
            if matches!(cell, Cell::Dense(id, _) if self.shapes.contains(id)) {
                continue;
            }
            if matches!(cell, Cell::CallbackSeen(key) | Cell::CallbackRunning(key) if evaluator.scopes.get(key.2).is_none())
            {
                continue;
            }
            evaluator.apply_cell(cell, &entry.joined);
            match cell {
                Cell::Binding(..) => bindings += 1,
                Cell::Dense(..) => dense += 1,
                _ => {}
            }
        }
        for id in &self.shapes {
            if evaluator.argument_objects[id]
                != [super::super::super::Value::Failure(
                    "conditional-collection-shape",
                )]
            {
                shape_cells += evaluator.argument_objects[id].len() as u64;
                evaluator.replace_dense_object(
                    *id,
                    vec![super::super::super::Value::Failure(
                        "conditional-collection-shape",
                    )],
                );
            }
        }
        for (cell, entry) in &self.entries {
            visited += 1;
            if let Cell::Disconnected(id, index) = cell {
                if entry.joined == Saved::Flag(true)
                    && !evaluator
                        .disconnected_argument_slots
                        .contains(&(*id, *index))
                {
                    evaluator.disconnect_mapped_slot(*id, *index);
                }
                evaluator.apply_cell(cell, &entry.joined);
            }
        }
        for (cell, entry) in &self.entries {
            visited += 1;
            if matches!(cell, Cell::Fresh(..)) {
                evaluator.apply_cell(cell, &entry.joined);
            }
        }
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.join_apply_cells_visited", visited);
            observer.increment("postgres.annotation.joined_binding_locations", bindings);
            observer.increment("postgres.annotation.joined_dense_locations", dense);
            observer.increment(
                "postgres.annotation.shape_dense_cells_replaced",
                shape_cells,
            );
        }
    }
}
