use super::{ConfigLocations, Kind, Node, Property};
use crate::fx::FxHashMap;
use granit_parser::{Event, Parser};
use std::collections::BTreeMap;

// Construction owns each mapping index once. No key scans remain in queries.
enum Frame {
    Mapping {
        id: usize,
        key: Option<usize>,
        properties: BTreeMap<String, Property>,
    },
    Sequence {
        id: usize,
        children: Vec<usize>,
    },
}

impl Frame {
    fn attach(&mut self, child: usize, locations: &ConfigLocations) {
        match self {
            Self::Mapping {
                key, properties, ..
            } => {
                if let Some(key_id) = key.take() {
                    let resolved = locations.resolve(key_id, &mut None);
                    if let Some(Kind::Scalar(text)) = resolved.map(|id| &locations.nodes[id].kind) {
                        properties.insert(
                            text.clone(),
                            Property {
                                key_line: locations.nodes[key_id].line,
                                value: child,
                            },
                        );
                    }
                } else {
                    *key = Some(child);
                }
            }
            Self::Sequence { children, .. } => children.push(child),
        }
    }

    fn finish(self) -> (usize, Kind) {
        match self {
            Self::Mapping { id, properties, .. } => (id, Kind::Mapping(properties)),
            Self::Sequence { id, children } => (id, Kind::Sequence(children)),
        }
    }
}

pub(super) fn parse(source: &str) -> Option<ConfigLocations> {
    let mut locations = ConfigLocations { nodes: Vec::new() };
    let mut anchors = FxHashMap::default();
    let mut stack: Vec<Frame> = Vec::new();
    let mut documents = 0;
    let options = granit_parser::options! { emit_comments: false };
    for event in Parser::new_from_str_with_options(source, options) {
        let (event, span) = event.ok()?;
        // Typed fields and serde_yaml::Value give tagged keys different identity.
        // Keep valid configs accepted without guessing which identity was used.
        if event.tag().is_some() {
            return None;
        }
        let (kind, anchor, container) = match event {
            Event::DocumentStart(..) => {
                documents += 1;
                if documents > 1 {
                    return None;
                }
                continue;
            }
            Event::MappingStart(_, anchor, _) => (Kind::Mapping(BTreeMap::new()), anchor, true),
            Event::SequenceStart(_, anchor, _) => (Kind::Sequence(Vec::new()), anchor, true),
            Event::Scalar(text, _, anchor, _) => (Kind::Scalar(text.into_owned()), anchor, false),
            Event::Alias(anchor) => (Kind::Alias(*anchors.get(&anchor)?), 0, false),
            Event::MappingEnd | Event::SequenceEnd => {
                let (id, kind) = stack.pop()?.finish();
                locations.nodes[id].kind = kind;
                continue;
            }
            _ => continue,
        };
        let id = locations.nodes.len();
        let mapping = matches!(kind, Kind::Mapping(_));
        locations.nodes.push(Node {
            line: span.start.line(),
            kind,
        });
        if anchor != 0 {
            anchors.insert(anchor, id);
        }
        if let Some(parent) = stack.last_mut() {
            parent.attach(id, &locations);
        }
        if container {
            stack.push(if mapping {
                Frame::Mapping {
                    id,
                    key: None,
                    properties: BTreeMap::new(),
                }
            } else {
                Frame::Sequence {
                    id,
                    children: Vec::new(),
                }
            });
        }
    }
    (!locations.nodes.is_empty() && stack.is_empty()).then_some(locations)
}
