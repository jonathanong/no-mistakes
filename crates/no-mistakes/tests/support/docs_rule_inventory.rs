use super::read;
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

struct Page {
    links: BTreeSet<String>,
    rule_titles: BTreeSet<String>,
}

pub(super) fn assert_inventory(root: &Path, source: &BTreeSet<String>) {
    assert!(
        !source.is_empty(),
        "source rule inventory must not be empty"
    );
    let failures = inventory_failures(root, source);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

pub(super) fn inventory_failures(root: &Path, source: &BTreeSet<String>) -> Vec<String> {
    let directory = root.join("docs/rules");
    let pages: BTreeMap<_, _> = std::fs::read_dir(&directory)
        .expect("read rule documentation directory")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "md"))
        .filter(|path| path.file_name().unwrap() != "README.md")
        .map(|path| {
            let id = path.file_stem().unwrap().to_str().unwrap().to_owned();
            (id, page(&read(&path)))
        })
        .collect();
    let guides = supporting_guides(source, &pages);
    let documented = pages
        .keys()
        .filter(|id| !guides.contains(*id))
        .cloned()
        .collect();
    let indexed = page(&read(&directory.join("README.md")))
        .links
        .difference(&guides)
        .cloned()
        .collect();
    let mut failures = Vec::new();
    for (label, expected, actual) in [
        ("missing rule page", source, &documented),
        ("extra rule page", &documented, source),
        ("missing README entry", source, &indexed),
        ("extra README entry", &indexed, source),
    ] {
        failures.extend(
            expected
                .difference(actual)
                .map(|id| format!("{label}: {id}")),
        );
    }
    for id in source {
        if pages
            .get(id)
            .is_some_and(|page| !page.rule_titles.contains(id))
        {
            failures.push(format!("rule page title mismatch: {id}"));
        }
    }
    failures
}

fn page(body: &str) -> Page {
    let mut links = BTreeSet::new();
    let mut heading = None;
    let mut rule_titles = BTreeSet::new();
    for event in Parser::new(body) {
        if let Event::Start(Tag::Link { dest_url, .. }) = &event {
            links.extend(local_page(dest_url).map(str::to_owned));
        }
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) => {
                heading = Some(Vec::new());
            }
            Event::End(TagEnd::Heading(HeadingLevel::H1)) => {
                if let Some([Event::Code(id)]) = heading.take().as_deref() {
                    rule_titles.insert(id.to_string());
                }
            }
            event => {
                if let Some(events) = heading.as_mut() {
                    events.push(event);
                }
            }
        }
    }
    Page { links, rule_titles }
}

fn local_page(target: &str) -> Option<&str> {
    let target = target.split(['#', '?']).next().unwrap();
    let target = target.strip_prefix("./").unwrap_or(target);
    let id = target.strip_suffix(".md")?;
    (id != "README" && !id.contains(['/', '\\', ':'])).then_some(id)
}

fn supporting_guides(
    source: &BTreeSet<String>,
    pages: &BTreeMap<String, Page>,
) -> BTreeSet<String> {
    let mut pending: Vec<_> = source
        .iter()
        .filter(|id| {
            pages
                .get(*id)
                .is_some_and(|page| page.rule_titles.contains(*id))
        })
        .cloned()
        .collect();
    let mut guides = BTreeSet::new();
    while let Some(id) = pending.pop() {
        for target in &pages[&id].links {
            // Linked SQL guides are not registry entries. A stale rule page
            // keeps its canonical title, so links cannot hide it as a guide.
            if pages
                .get(target)
                .is_some_and(|page| page.rule_titles.is_empty())
                && !source.contains(target)
                && guides.insert(target.clone())
            {
                pending.push(target.clone());
            }
        }
    }
    guides
}
