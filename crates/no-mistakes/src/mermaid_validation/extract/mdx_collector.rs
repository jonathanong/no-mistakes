use std::ops::Range;

use super::{html_fallback, MermaidFenceCollector};

impl MermaidFenceCollector<'_> {
    pub(super) fn advance_mdx_expression_to(&mut self, end: usize) {
        let end = end.min(self.source.len());
        if end > self.mdx_scanned_until {
            // Pulldown-cmark does not understand top-level MDX expressions or
            // ESM. Scan parser gaps so fence-looking text in their multiline
            // JavaScript values is not mistaken for Markdown.
            self.mdx_expression
                .observe_source(&self.source.as_bytes()[self.mdx_scanned_until..end]);
            self.mdx_scanned_until = end;
        }
    }

    pub(super) fn recover_overlapping_mdx_code_block(&mut self, range: Range<usize>) -> bool {
        if !self.mdx_scanning_enabled || range.start >= self.mdx_scanned_until {
            return false;
        }
        let extracted = html_fallback::extract(
            self.source,
            self.mdx_scanned_until..range.end,
            &mut self.mdx_expression,
        );
        self.fences.extend(extracted.fences);
        self.mdx_scanned_until = range.end.max(extracted.consumed_until);
        true
    }
}
