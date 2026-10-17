use super::summary::ImportedSite;

/// Imported-helper calls seen during one walk. A call expression can be
/// recovered more than once (provenance, composition, alternatives), so each
/// distinct shape at one source offset is interned to a single hole index.
#[derive(Default)]
pub(crate) struct Sites {
    list: Vec<ImportedSite>,
    starts: Vec<u32>,
}

impl Sites {
    pub(crate) fn intern(&mut self, start: u32, site: ImportedSite) -> usize {
        let existing = self
            .list
            .iter()
            .zip(&self.starts)
            .position(|(known, at)| *at == start && *known == site);
        existing.unwrap_or_else(|| {
            self.list.push(site);
            self.starts.push(start);
            self.list.len() - 1
        })
    }

    pub(crate) fn into_list(self) -> Vec<ImportedSite> {
        self.list
    }
}
