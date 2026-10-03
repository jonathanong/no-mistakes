use super::{Scan, SqlBareRead};
use std::ops::ControlFlow;

impl Scan {
    pub(super) fn finish_frame(&mut self) -> ControlFlow<()> {
        let frame = self.stack.pop().unwrap_or_default();
        self.ctes = frame.previous_ctes;
        let up: Vec<Vec<String>> = frame
            .qualifiers
            .into_iter()
            .filter(|qualifier| !frame.relations.contains(qualifier))
            .collect();
        // A bare name that is a relation's own is a whole-row reference, not a column.
        let own = frame.bare.iter().filter(|(name, count)| {
            **count > frame.labels.get(*name).copied().unwrap_or(0)
                && !frame.whole_rows.contains(*name)
        });
        let mut reads: Vec<SqlBareRead> = own
            .map(|(name, _)| SqlBareRead {
                column: name.clone(),
                tables: Vec::new(),
            })
            .chain(frame.reads)
            .collect();
        if frame.foreign {
            reads.clear();
        } else {
            reads.retain(|read| !frame.columns.contains(&read.column));
        }
        for read in &mut reads {
            read.tables.extend(frame.tables.iter().cloned());
            read.tables.sort();
            read.tables.dedup();
        }
        match self.stack.last_mut() {
            Some(parent) => {
                parent.qualifiers.extend(up);
                parent.reads.extend(reads);
            }
            None => {
                self.unresolved.extend(up);
                self.reads = reads;
            }
        }
        ControlFlow::Continue(())
    }
}
