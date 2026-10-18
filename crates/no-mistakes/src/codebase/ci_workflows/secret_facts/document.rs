use granit_parser::{Event, Parser};
use std::collections::BTreeMap;

pub(super) enum Kind {
    Mapping(Vec<usize>),
    Sequence(Vec<usize>),
    Scalar(String),
    Alias(usize),
}

pub(super) struct Node {
    pub(super) kind: Kind,
    pub(super) line: usize,
    pub(super) raw: String,
}

pub(super) struct Document {
    pub(super) nodes: Vec<Node>,
}

impl Document {
    // Supplemental markers locate authored values; serde_yaml remains the
    // authority for their types. This projection is retained by the request.
    pub(super) fn parse(source: &str) -> Option<Self> {
        let mut result = Self { nodes: Vec::new() };
        let mut anchors = BTreeMap::new();
        let mut stack = Vec::<usize>::new();
        for item in Parser::new_from_str(source) {
            let (event, span) = item.ok()?;
            let (kind, anchor, container) = match event {
                Event::MappingStart(_, anchor, _) => (Kind::Mapping(Vec::new()), anchor, true),
                Event::SequenceStart(_, anchor, _) => (Kind::Sequence(Vec::new()), anchor, true),
                Event::Scalar(text, _, anchor, _) => {
                    (Kind::Scalar(text.into_owned()), anchor, false)
                }
                Event::Alias(anchor) => (Kind::Alias(*anchors.get(&anchor)?), 0, false),
                Event::MappingEnd | Event::SequenceEnd => {
                    stack.pop();
                    continue;
                }
                _ => continue,
            };
            let id = result.nodes.len();
            result.nodes.push(Node {
                kind,
                line: span.tag_start().unwrap_or(span.start).line(),
                raw: span.slice(source).unwrap_or("").to_owned(),
            });
            if anchor != 0 {
                anchors.insert(anchor, id);
            }
            if let Some(&parent) = stack.last() {
                if let Kind::Mapping(children) | Kind::Sequence(children) =
                    &mut result.nodes[parent].kind
                {
                    children.push(id);
                }
            }
            if container {
                stack.push(id);
            }
        }
        (!result.nodes.is_empty()).then_some(result)
    }

    pub(super) fn resolved(&self, id: usize) -> &Node {
        match self.nodes[id].kind {
            Kind::Alias(target) => &self.nodes[target],
            _ => &self.nodes[id],
        }
    }

    pub(super) fn mapping_value(&self, id: usize, key: &str) -> Option<usize> {
        let Kind::Mapping(children) = &self.resolved(id).kind else {
            return None;
        };
        children.as_chunks::<2>().0.iter().find_map(|pair| {
            matches!(&self.resolved(pair[0]).kind, Kind::Scalar(text) if text == key)
                .then_some(pair[1])
        })
    }

    pub(super) fn line(&self, id: Option<usize>) -> usize {
        id.map_or(1, |id| self.nodes[id].line)
    }
}
