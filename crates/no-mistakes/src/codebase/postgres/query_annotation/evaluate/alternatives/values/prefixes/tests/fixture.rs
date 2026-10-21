use super::super::{Arena, Value};
use crate::codebase::postgres::query_annotation::evaluate::value::Values;
use crate::fx::{FxHashMap, FxHashSet};
use serde_json::Value as Json;
use std::collections::BTreeMap;

pub(super) struct Case {
    pub root: Value,
    pub objects: FxHashMap<u64, Vec<Value>>,
    pub extras: FxHashMap<u64, BTreeMap<usize, Value>>,
    pub definite: FxHashSet<(u64, usize)>,
    pub expected: BTreeMap<u64, (String, bool)>,
}

fn value(json: &Json, nodes: &[Values]) -> Value {
    let parts = json.as_array().unwrap();
    match parts[0].as_str().unwrap() {
        "prefix" => Value::Prefix(
            parts[2].as_str().unwrap().into(),
            parts[3].as_bool().unwrap(),
            parts[1].as_u64(),
        ),
        "primitive" => Value::Primitive,
        "arguments" => Value::Arguments(parts[1].as_u64().unwrap()),
        "array" => Value::Array(parts[1].as_u64().unwrap()),
        "promise" => Value::Promise(Box::new(value(&parts[1], nodes))),
        "evaluated" => Value::Evaluated(Box::new(value(&parts[1], nodes)), true),
        "record" => Value::Record(
            parts[1]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| value(x, nodes))
                .collect::<Vec<_>>()
                .into(),
            Vec::new(),
        ),
        kind => {
            let values = nodes[parts[1].as_u64().unwrap() as usize].clone();
            match kind {
                "aggregate" => Value::Aggregate(values),
                "references" => Value::References(values),
                "object" => Value::Object(values),
                "node" | "record_ref" => Value::Record(values, Vec::new()),
                "joined" => Value::Joined(values),
                "possible" => Value::Possible(values),
                _ => panic!("unknown saved fixture value: {kind}"),
            }
        }
    }
}

pub(super) fn load(name: &str) -> Case {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-shared-record-dag/prefix-semantics")
        .join(format!("{name}.json"));
    let json: Json = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut nodes: Vec<Values> = Vec::new();
    for node in json["nodes"].as_array().unwrap() {
        nodes.push(
            node.as_array()
                .unwrap()
                .iter()
                .map(|x| value(x, &nodes))
                .collect::<Vec<_>>()
                .into(),
        );
    }
    Case {
        root: value(&json["root"], &nodes),
        objects: json
            .get("objects")
            .and_then(Json::as_object)
            .into_iter()
            .flatten()
            .map(|(id, values)| {
                (
                    id.parse().unwrap(),
                    values
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|x| value(x, &nodes))
                        .collect(),
                )
            })
            .collect(),
        extras: json
            .get("extras")
            .and_then(Json::as_object)
            .into_iter()
            .flatten()
            .map(|(id, slots)| {
                (
                    id.parse().unwrap(),
                    slots
                        .as_object()
                        .unwrap()
                        .iter()
                        .map(|(index, x)| (index.parse().unwrap(), value(x, &nodes)))
                        .collect(),
                )
            })
            .collect(),
        definite: json
            .get("definite")
            .and_then(Json::as_array)
            .into_iter()
            .flatten()
            .map(|pair| {
                (
                    pair[0].as_u64().unwrap(),
                    pair[1].as_u64().unwrap() as usize,
                )
            })
            .collect(),
        expected: serde_json::from_value(json["expected"].clone()).unwrap(),
    }
}

impl Case {
    pub fn arena(&self) -> Arena<'_> {
        Arena {
            objects: &self.objects,
            extras: &self.extras,
        }
    }
}
