use super::merge_keys::{expand_located, KeyLocation};
use crate::fx::FxHashMap;
use granit_parser::{Event, Parser, ScalarStyle};
use serde_yaml::Value;

#[derive(Default)]
pub(super) struct MergeLocations {
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

impl MergeLocations {
    pub(super) fn parse(source: &str) -> Self {
        let options = granit_parser::options! { emit_comments: false };
        let mut locations = Self::default();
        let mut anchors = FxHashMap::default();
        let mut stack: Vec<usize> = Vec::new();
        for event in Parser::new_from_str_with_options(source, options) {
            let Ok((event, span)) = event else {
                break;
            };
            let line = span.start.line();
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

    pub(super) fn expand(&self, index: usize, value: &mut Value) -> Result<(), (usize, String)> {
        let root = self.roots.get(index).copied().flatten();
        let fallback_line = root.map_or(1, |root| self.nodes[root].line);
        let locations =
            root.map_or_else(FxHashMap::default, |root| self.key_locations(root, value));
        expand_located(value, &|key| {
            locations.get(&key.as_ptr()).copied().ok_or_else(|| {
                (
                    fallback_line,
                    "could not match a YAML `<<` key to its source location".to_string(),
                )
            })
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
    fn key_locations(&self, root: usize, value: &Value) -> FxHashMap<*const u8, KeyLocation> {
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
                            if let Some(location) = self.key_location(key_mark) {
                                lines.insert(key.as_ptr(), location);
                            }
                        }
                        pending.push((value_mark, value));
                    }
                }
                (Kind::Sequence(children), Value::Sequence(items))
                    if children.len() == items.len() =>
                {
                    pending.extend(children.iter().copied().zip(items));
                }
                _ => {}
            }
        }
        lines
    }
}
