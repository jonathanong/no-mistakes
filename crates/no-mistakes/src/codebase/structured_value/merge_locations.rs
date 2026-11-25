use super::document_spans::{document_spans, DocumentSpan};
use super::merge_keys::{expand_located, KeyLocation, MergeError};
use crate::fx::FxHashMap;
use granit_parser::{Event, Parser, ScalarStyle};
use serde_yaml::Value;

pub(super) struct MergeLocations<'a> {
    source: &'a str,
    doc_spans: Vec<DocumentSpan>,
    roots: Vec<Option<usize>>,
    nodes: Vec<Node>,
}

struct Node {
    line: usize,
    kind: Kind,
}

enum Kind {
    Scalar,
    MergeKey,
    LiteralMergeKey,
    Mapping(Vec<usize>),
    Sequence(Vec<usize>),
    Alias(usize),
}

impl<'a> MergeLocations<'a> {
    pub(super) fn parse(source: &'a str) -> Self {
        Self::parse_with_offset(source, 0)
    }

    fn parse_with_offset(source: &'a str, line_offset: usize) -> Self {
        let options = granit_parser::options! { emit_comments: false };
        let mut locations = Self {
            source,
            doc_spans: document_spans(source),
            roots: Vec::new(),
            nodes: Vec::new(),
        };
        let mut anchors = FxHashMap::default();
        let mut stack: Vec<usize> = Vec::new();
        for event in Parser::new_from_str_with_options(source, options) {
            let Ok((event, span)) = event else {
                break;
            };
            let line = span.start.line() + line_offset;
            let (anchor, kind) = match event {
                Event::DocumentStart(..) => {
                    locations.roots.push(None);
                    anchors.clear();
                    continue;
                }
                Event::MappingStart(_, anchor, _) => (anchor, Kind::Mapping(Vec::new())),
                Event::SequenceStart(_, anchor, _) => (anchor, Kind::Sequence(Vec::new())),
                Event::MappingEnd | Event::SequenceEnd => {
                    stack.pop();
                    continue;
                }
                Event::Scalar(text, ScalarStyle::Plain, anchor, None) if text == "<<" => {
                    (anchor, Kind::MergeKey)
                }
                Event::Scalar(text, _, anchor, _) if text == "<<" => {
                    (anchor, Kind::LiteralMergeKey)
                }
                Event::Scalar(_, _, anchor, _) => (anchor, Kind::Scalar),
                // The parser emits Alias only after emitting its anchor node.
                Event::Alias(anchor) => (0, Kind::Alias(anchors[&anchor])),
                _ => continue,
            };
            let collection = matches!(kind, Kind::Mapping(_) | Kind::Sequence(_));
            let id = locations.nodes.len();
            if anchor != 0 {
                anchors.insert(anchor, id);
            }
            if let Some(&parent) = stack.last() {
                match &mut locations.nodes[parent].kind {
                    Kind::Mapping(children) | Kind::Sequence(children) => children.push(id),
                    _ => unreachable!("only collections enter the YAML parent stack"),
                }
            } else {
                *locations.roots.last_mut().expect("YAML document start") = Some(id);
            }
            locations.nodes.push(Node { line, kind });
            if collection {
                stack.push(id);
            }
        }
        locations
    }

    pub(super) fn expand(&self, index: usize, value: &mut Value) -> Result<(), MergeError> {
        let doc_span = self.doc_spans.get(index).copied();
        let doc_source = doc_span.map_or(self.source, |span| {
            &self.source[span.start_byte..span.end_byte]
        });
        let root = self.roots.get(index).copied().flatten();
        if root.is_none() {
            if let Some(span) = doc_span {
                if span.start_byte > 0 {
                    let reparsed = Self::parse_with_offset(doc_source, span.start_line - 1);
                    if reparsed.roots.first().copied().flatten().is_some() {
                        return reparsed.expand(0, value);
                    }
                }
            }
        }
        let fallback_line = root
            .map(|root| self.nodes[root].line)
            .or_else(|| doc_span.map(|span| span.start_line))
            .unwrap_or(1);
        let locations = root
            .and_then(|root| self.key_locations(root, value))
            .ok_or(MergeError::MissingLocation(fallback_line))?;
        expand_located(value, &|key| {
            locations
                .get(&key.as_ptr())
                .copied()
                .ok_or(MergeError::LocationChanged(fallback_line))
        })
    }

    fn key_location(&self, key: usize) -> Option<KeyLocation> {
        let node = &self.nodes[key];
        let kind = match node.kind {
            Kind::Alias(anchor) => &self.nodes[anchor].kind,
            _ => &node.kind,
        };
        match kind {
            Kind::MergeKey => Some(KeyLocation::Merge(node.line)),
            Kind::LiteralMergeKey => Some(KeyLocation::Literal),
            _ => None,
        }
    }

    // Expansion moves YAML keys in place. A lost string identity is an error
    // because only a key matched to a literal source node can be left intact.
    fn key_locations(
        &self,
        root: usize,
        value: &Value,
    ) -> Option<FxHashMap<*const u8, KeyLocation>> {
        let mut lines = FxHashMap::default();
        let mut pending = vec![(root, value)];
        while let Some((id, value)) = pending.pop() {
            if let Value::Tagged(tagged) = value {
                pending.push((id, &tagged.value));
                continue;
            }
            match (&self.nodes[id].kind, value) {
                (Kind::Alias(target), value) => pending.push((*target, value)),
                (Kind::Mapping(children), Value::Mapping(mapping))
                    if children.len() == mapping.len() * 2 =>
                {
                    let (marks, _) = children.as_chunks::<2>();
                    for ((key, value), &[key_mark, value_mark]) in mapping.iter().zip(marks) {
                        if let Some(key) = key.as_str().filter(|key| *key == "<<") {
                            lines.insert(key.as_ptr(), self.key_location(key_mark)?);
                        }
                        pending.push((value_mark, value));
                    }
                }
                (Kind::Sequence(children), Value::Sequence(items))
                    if children.len() == items.len() =>
                {
                    pending.extend(children.iter().copied().zip(items));
                }
                (_, Value::Mapping(_) | Value::Sequence(_)) if super::contains_merge(value) => {
                    return None;
                }
                _ => {}
            }
        }
        Some(lines)
    }
}
