use super::super::scan::{Gap, PARSE_GAP, ROUTINE_GAP};
use super::{EmbeddedSqlCall, Positions, SqlStatementFileFacts};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct AlternativeGaps {
    counts: BTreeMap<Gap, usize>,
}

impl AlternativeGaps {
    pub(super) fn record(
        &mut self,
        call: &EmbeddedSqlCall,
        variant_index: usize,
        statement: &SqlStatementFileFacts,
    ) -> bool {
        if call
            .variants
            .get(variant_index)
            .is_some_and(|variant| variant.variants_truncated)
        {
            self.counts.insert(
                (
                    call.line as usize,
                    "executor SQL alternatives are incomplete",
                ),
                1,
            );
            return true;
        }
        let parse_gaps = statement.parse_failure_sites().map(|(site, line)| {
            let imported = statement
                .variant_locations
                .as_ref()
                .and_then(|locations| locations.position(site))
                .is_some_and(|position| position.source_path.is_some());
            let line = if imported { call.line as usize } else { line };
            (line, PARSE_GAP)
        });
        let mut positions = None;
        let routine_gaps = statement.lock_inventory_unanalyzed.iter().map(|&line| {
            let (_, line, _) = positions
                .get_or_insert_with(|| Positions::new(call, variant_index))
                .source(line, 1);
            (line, ROUTINE_GAP)
        });
        let mut counts = BTreeMap::new();
        for gap in parse_gaps.chain(routine_gaps) {
            *counts.entry(gap).or_insert(0) += 1;
        }
        // Alternative versions describe the same executor call and share its gaps.
        for (gap, count) in counts {
            let previous = self.counts.entry(gap).or_default();
            *previous = (*previous).max(count);
        }
        statement.parse_failed || !statement.lock_inventory_unanalyzed.is_empty()
    }

    pub(super) fn append_to(self, gaps: &mut Vec<Gap>) {
        for (gap, count) in self.counts {
            gaps.extend(std::iter::repeat_n(gap, count));
        }
    }
}
