use super::{event, outputs_mode, Value};

fn values(file: &str) -> Vec<(String, Value)> {
    outputs_mode("coverage-evaluator", file, true)
}

fn describe(value: &Value) -> String {
    match value {
        Value::Prefix(text, complete, _) => format!("prefix {text:?}, complete={complete}"),
        Value::Number(number) => format!("number {number}"),
        Value::Boolean(value) => format!("boolean {value}"),
        Value::Failure(reason) => format!("failure {reason}"),
        Value::Unknown => "unknown".into(),
        Value::Primitive => "primitive".into(),
        Value::Joined(values) => format!(
            "joined {}",
            values.iter().map(describe).collect::<Vec<_>>().join(", ")
        ),
        Value::Aggregate(values) => format!(
            "aggregate {}",
            values.iter().map(describe).collect::<Vec<_>>().join(", ")
        ),
        _ => "other value".into(),
    }
}

include!("evaluator_behavior_part1.rs");

include!("evaluator_behavior_part2.rs");
