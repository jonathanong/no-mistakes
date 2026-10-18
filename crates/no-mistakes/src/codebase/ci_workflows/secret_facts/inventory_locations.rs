use super::document::{Document, Kind};
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct InventoryLocations {
    lines: BTreeMap<(String, String), Vec<usize>>,
}

impl InventoryLocations {
    pub(crate) fn parse(source: &str) -> Self {
        let mut result = Self::default();
        if let Some(document) = Document::parse(source) {
            for node in &document.nodes {
                let Kind::Mapping(children) = &node.kind else {
                    continue;
                };
                for pair in children.as_chunks::<2>().0 {
                    if matches!(&document.resolved(pair[0]).kind, Kind::Scalar(key) if key == "secretInventory")
                    {
                        result.inventory(&document, pair[1]);
                    }
                }
            }
        }
        result
    }

    pub(crate) fn line(&self, environment: &str, name: &str) -> Option<usize> {
        self.lines
            .get(&(environment.to_owned(), name.to_owned()))
            .filter(|lines| lines.len() == 1)
            .map(|lines| lines[0])
    }

    fn inventory(&mut self, document: &Document, node: usize) {
        if let Some(repository) = document.mapping_value(node, "repository") {
            self.names(document, repository, "");
        }
        if let Some(environments) = document.mapping_value(node, "environments") {
            if let Kind::Mapping(children) = &document.resolved(environments).kind {
                for pair in children.as_chunks::<2>().0 {
                    if let Kind::Scalar(environment) = &document.resolved(pair[0]).kind {
                        self.names(document, pair[1], &environment.to_ascii_lowercase());
                    }
                }
            }
        }
    }

    fn names(&mut self, document: &Document, node: usize, environment: &str) {
        if let Kind::Sequence(children) = &document.resolved(node).kind {
            for &child in children {
                if let Kind::Scalar(name) = &document.resolved(child).kind {
                    self.lines
                        .entry((environment.to_owned(), name.to_ascii_lowercase()))
                        .or_default()
                        .push(document.nodes[child].line);
                }
            }
        }
    }
}
