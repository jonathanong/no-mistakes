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

#[cfg(test)]
mod tests;
