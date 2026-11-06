use crate::codebase::rules::github_actions_workflow_files::job_lines::document::Document;
use std::collections::BTreeMap;

pub(super) fn collect(source: &str) -> BTreeMap<String, BTreeMap<usize, usize>> {
    let mut locations = BTreeMap::new();
    let Some(document) = Document::parse(source) else {
        return locations;
    };
    let Some(jobs) = document
        .field(0, "jobs")
        .and_then(|id| document.mapping(id))
    else {
        return locations;
    };
    for pair in jobs.as_chunks::<2>().0 {
        let Some(job) = document.key(pair[0]) else {
            continue;
        };
        let Some(steps) = document
            .field(pair[1], "steps")
            .and_then(|id| document.sequence(id))
        else {
            continue;
        };
        let lines = steps
            .iter()
            .enumerate()
            .filter_map(|(index, &step)| {
                document
                    .field_line(step, "uses")
                    .map(|line| (index + 1, line))
            })
            .collect();
        locations.insert(job.to_owned(), lines);
    }
    locations
}
