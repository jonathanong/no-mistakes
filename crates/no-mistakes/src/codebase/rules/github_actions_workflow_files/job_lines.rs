use std::collections::BTreeMap;

mod document;
use document::Document;

pub(crate) struct Locations {
    lines: BTreeMap<String, usize>,
    declaration: usize,
    permissions: usize,
}

impl Locations {
    pub(crate) fn line(&self, job: &str) -> usize {
        self.lines.get(job).copied().unwrap_or(self.declaration)
    }

    pub(crate) fn permissions_line(&self) -> usize {
        self.permissions
    }

    pub(crate) fn jobs_line(&self) -> usize {
        self.declaration
    }
}

// Supplemental source locations share the already-read workflow text. The primary
// serde_yaml value remains authoritative for job/timeout semantics.
pub(crate) fn collect(source: &str) -> Locations {
    let mut locations = Locations {
        lines: BTreeMap::new(),
        declaration: 1,
        permissions: 1,
    };
    let Some(document) = Document::parse(source) else {
        return locations;
    };
    let Some(root) = document.mapping(0) else {
        return locations;
    };
    for pair in root.as_chunks::<2>().0 {
        if document.key(pair[0]) == Some("permissions") {
            locations.permissions = document.nodes[pair[0]].line;
        }
        if document.key(pair[0]) != Some("jobs") {
            continue;
        }
        locations.declaration = document.nodes[pair[0]].line;
        if let Some(jobs) = document.mapping(pair[1]) {
            for job in jobs.as_chunks::<2>().0 {
                if let Some(key) = document.key(job[0]) {
                    locations
                        .lines
                        .insert(key.to_owned(), document.nodes[job[0]].line);
                }
            }
        }
    }
    locations
}
