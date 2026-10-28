use granit_parser::{Event, Parser, ScalarStyle, Tag};
use std::collections::BTreeMap;

pub(super) enum Kind {
    Mapping(Vec<usize>),
    String(String),
    Alias(usize),
    Other,
}

pub(super) struct Node {
    pub(super) kind: Kind,
    pub(super) line: usize,
}

pub(super) struct Document {
    pub(super) nodes: Vec<Node>,
    anchors: BTreeMap<usize, usize>,
}

impl Document {
    pub(super) fn parse(source: &str) -> Option<Self> {
        let mut document = Self {
            nodes: Vec::new(),
            anchors: BTreeMap::new(),
        };
        let mut stack: Vec<usize> = Vec::new();
        for event in Parser::new_from_str(source) {
            let (event, span) = event.ok()?;
            let is_key = stack.last().is_some_and(|&parent| {
                matches!(&document.nodes[parent].kind, Kind::Mapping(children) if children.len() % 2 == 0)
            });
            let (kind, anchor, container) = match event {
                Event::MappingStart(_, anchor, _) => (Kind::Mapping(Vec::new()), anchor, true),
                Event::SequenceStart(_, anchor, _) => (Kind::Other, anchor, true),
                Event::Scalar(text, style, anchor, tag) => {
                    let key = (is_key || anchor != 0)
                        .then(|| scalar_key(&text, style, tag.as_deref()))
                        .flatten();
                    (key.map_or(Kind::Other, Kind::String), anchor, false)
                }
                // The parser has already resolved this alias to a preceding anchor.
                Event::Alias(anchor) => (Kind::Alias(document.anchors[&anchor]), 0, false),
                Event::MappingEnd | Event::SequenceEnd => {
                    stack.pop();
                    continue;
                }
                _ => continue,
            };
            let id = document.nodes.len();
            let line = span.tag_start().unwrap_or(span.start).line();
            document.nodes.push(Node { kind, line });
            if anchor != 0 {
                document.anchors.insert(anchor, id);
            }
            if let Some(&parent) = stack.last() {
                if let Kind::Mapping(children) = &mut document.nodes[parent].kind {
                    children.push(id);
                }
            }
            if container {
                stack.push(id);
            }
        }
        Some(document)
    }

    fn resolved(&self, id: usize) -> &Node {
        let node = &self.nodes[id];
        match node.kind {
            // YAML aliases cannot themselves declare anchors, so one hop suffices.
            Kind::Alias(target) => &self.nodes[target],
            _ => node,
        }
    }

    pub(super) fn mapping(&self, id: usize) -> Option<&[usize]> {
        match &self.nodes.get(id).map(|_| self.resolved(id))?.kind {
            Kind::Mapping(children) => Some(children),
            _ => None,
        }
    }

    pub(super) fn key(&self, id: usize) -> Option<&str> {
        match &self.resolved(id).kind {
            Kind::String(key) => Some(key),
            _ => None,
        }
    }
}

fn scalar_key(text: &str, style: ScalarStyle, tag: Option<&Tag>) -> Option<String> {
    if let Some(tag) = tag {
        match tag.core_suffix() {
            Some("bool" | "int" | "float" | "null") => return None,
            Some(_) => return Some(text.to_owned()),
            None => {}
        }
        // serde_yaml unwraps local custom tags and preserves implicit scalar types;
        // unknown global tags instead deserialize their scalar payload as strings.
        if tag.handle().chars().chain(tag.suffix().chars()).next() != Some('!') {
            return Some(text.to_owned());
        }
    }
    if style != ScalarStyle::Plain {
        return Some(text.to_owned());
    }
    serde_yaml::from_str::<serde_yaml::Value>(text)
        .ok()
        .and_then(|key| key.as_str().map(str::to_owned))
}
