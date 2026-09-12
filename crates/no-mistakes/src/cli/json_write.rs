use serde::Serialize;
use std::io::{self, Write};

/// Serialize JSON to locked stdout without an intermediate `String`.
pub fn print_json<T: Serialize + ?Sized>(value: &T) {
    let stdout = io::stdout();
    write_json(&mut stdout.lock(), value);
}

pub(super) fn write_json<W: Write>(out: &mut W, value: &(impl Serialize + ?Sized)) {
    serde_json::to_writer(&mut *out, value).expect("serialization of Rust structs never fails");
    out.write_all(b"\n")
        .expect("writing JSON newline to stdout never fails");
}

pub(crate) fn json_string<T: Serialize + ?Sized>(value: &T) -> String {
    serde_json::to_string(value).expect("JSON serialization of Rust structs never fails")
}

pub(crate) fn json_value<T: Serialize + ?Sized>(value: &T) -> serde_json::Value {
    serde_json::to_value(value).expect("JSON serialization of Rust structs never fails")
}

pub(crate) fn json_pretty<T: Serialize + ?Sized>(value: &T) -> String {
    serde_json::to_string_pretty(value).expect("JSON serialization of Rust structs never fails")
}

pub(crate) fn yaml_string<T: Serialize + ?Sized>(value: &T) -> String {
    serde_yaml::to_string(value).expect("YAML serialization of Rust structs never fails")
}

#[cfg(test)]
mod tests;
