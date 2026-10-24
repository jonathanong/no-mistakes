//! Best-effort authored locations. This metadata never decides config validity.
use std::collections::BTreeMap;
use std::path::Path;

mod json;
mod yaml;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Segment {
    Key(String),
    Index(usize),
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) struct ConfigPath(Vec<Segment>);

impl ConfigPath {
    pub(crate) fn key(mut self, key: impl Into<String>) -> Self {
        self.0.push(Segment::Key(key.into()));
        self
    }

    pub(crate) fn index(mut self, index: usize) -> Self {
        self.0.push(Segment::Index(index));
        self
    }

    pub(crate) fn as_slice(&self) -> &[Segment] {
        &self.0
    }
}

pub(crate) struct ConfigLocations {
    nodes: Vec<Node>,
}

struct Node {
    line: usize,
    kind: Kind,
}

struct Property {
    key_line: usize,
    value: usize,
}

enum Kind {
    Scalar(String),
    Mapping(BTreeMap<String, Property>),
    Sequence(Vec<usize>),
    Alias(usize),
}

impl ConfigLocations {
    pub(crate) fn parse(source: &str, path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "yaml" | "yml" => yaml::parse(source),
            "json" => json::parse(source, false),
            "jsonc" => json::parse(source, true),
            _ => None,
        }
    }

    pub(crate) fn line(&self, path: &ConfigPath) -> Option<usize> {
        let mut id = 0;
        let mut line = self.nodes.first()?.line;
        let mut alias_line = None;
        for segment in path.as_slice() {
            id = self.resolve(id, &mut alias_line)?;
            let node = &self.nodes[id];
            match (segment, &node.kind) {
                (Segment::Key(key), Kind::Mapping(properties)) => {
                    let property = properties.get(key)?;
                    line = property.key_line;
                    id = property.value;
                }
                (Segment::Index(index), Kind::Sequence(children)) => {
                    id = *children.get(*index)?;
                    line = self.nodes[id].line;
                }
                // StringOrList deserialization exposes a scalar as its first item.
                (Segment::Index(0), Kind::Scalar(_)) => line = node.line,
                _ => return None,
            }
        }
        self.resolve(id, &mut alias_line)?;
        Some(alias_line.unwrap_or(line))
    }

    fn resolve(&self, mut id: usize, alias_line: &mut Option<usize>) -> Option<usize> {
        // Bound alias traversal even for recursive anchors; never expand subtrees.
        for _ in 0..self.nodes.len() {
            let node = self.nodes.get(id)?;
            if let Kind::Alias(target) = node.kind {
                alias_line.get_or_insert(node.line);
                id = target;
            } else {
                return Some(id);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests;
