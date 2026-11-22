use super::super::scan::{record_parse_failures, Gap};
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
        let mut gaps = Vec::new();
        let mut positions = None;
        let incomplete = record_parse_failures(statement, &mut gaps, |line| {
            positions
                .get_or_insert_with(|| Positions::new(call, variant_index))
                .source(line, 1)
                .1
        });
        let mut counts = BTreeMap::new();
        for gap in gaps {
            *counts.entry(gap).or_insert(0) += 1;
        }
        // Alternative versions describe the same executor call and share its gaps.
        for (gap, count) in counts {
            let previous = self.counts.entry(gap).or_default();
            *previous = (*previous).max(count);
        }
        incomplete
    }

    pub(super) fn append_to(self, gaps: &mut Vec<Gap>) {
        for (gap, count) in self.counts {
            gaps.extend(std::iter::repeat_n(gap, count));
        }
    }
}
