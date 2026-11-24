use super::merge_keys::expand_located;
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
        let lines = self
            .roots
            .get(index)
            .copied()
            .flatten()
            .map_or_else(FxHashMap::default, |root| self.key_lines(root, value));
        expand_located(value, &|key| {
            key.as_str()
                .and_then(|key| lines.get(&key.as_ptr()).copied())
        })
    }

    fn merge_line(&self, key: usize) -> Option<usize> {
        let node = &self.nodes[key];
        match node.kind {
            Kind::MergeKey => Some(node.line),
            Kind::Alias(anchor) if matches!(self.nodes[anchor].kind, Kind::MergeKey) => {
                Some(node.line)
            }
            _ => None,
        }
    }

    // Expansion moves YAML keys in place. Their string allocations retain
    // their identity even when a merge moves an entry into a different map.
    fn key_lines(&self, root: usize, value: &Value) -> FxHashMap<*const u8, usize> {
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
                            if let Some(line) = self.merge_line(key_mark) {
                                lines.insert(key.as_ptr(), line);
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
