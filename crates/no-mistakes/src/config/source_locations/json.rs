use super::{ConfigLocations, Kind, Node, Property};
use jsonc_parser::{ast::Value, common::Ranged};
use std::collections::BTreeMap;

pub(super) fn parse(source: &str, comments: bool) -> Option<ConfigLocations> {
    let mut options = crate::config::jsonc_parse_options();
    options.allow_comments = comments;
    options.allow_trailing_commas = comments;
    let value = jsonc_parser::parse_to_ast(source, &Default::default(), &options)
        .ok()?
        .value?;
    let newlines: Vec<_> = source
        .match_indices('\n')
        .map(|(offset, _)| offset)
        .collect();
    let mut locations = ConfigLocations { nodes: Vec::new() };
    append(value, &newlines, &mut locations.nodes);
    Some(locations)
}

fn line(newlines: &[usize], offset: usize) -> usize {
    newlines.partition_point(|&newline| newline < offset) + 1
}

fn append(value: Value<'_>, newlines: &[usize], nodes: &mut Vec<Node>) -> usize {
    let id = nodes.len();
    nodes.push(Node {
        line: line(newlines, value.start()),
        kind: Kind::Scalar(String::new()),
    });
    let kind = match value {
        Value::Object(object) => {
            let mut properties = BTreeMap::new();
            for property in object.properties {
                let key_line = line(newlines, property.name.start());
                let value = append(property.value, newlines, nodes);
                properties.insert(property.name.into_string(), Property { key_line, value });
            }
            Kind::Mapping(properties)
        }
        Value::Array(array) => Kind::Sequence(
            array
                .elements
                .into_iter()
                .map(|value| append(value, newlines, nodes))
                .collect(),
        ),
        Value::StringLit(text) => Kind::Scalar(text.value.into_owned()),
        _ => Kind::Scalar(String::new()),
    };
    nodes[id].kind = kind;
    id
}
