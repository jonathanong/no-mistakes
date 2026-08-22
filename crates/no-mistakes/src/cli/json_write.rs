use serde::Serialize;
use std::io::{self, Write};

/// Serialize JSON to locked stdout without an intermediate `String`.
pub fn print_json<T: Serialize + ?Sized>(value: &T) {
    let stdout = io::stdout();
    write_json(&mut stdout.lock(), value);
}

fn write_json<W: Write>(out: &mut W, value: &(impl Serialize + ?Sized)) {
    serde_json::to_writer(&mut *out, value).expect("serialization of Rust structs never fails");
    out.write_all(b"\n")
        .expect("writing JSON newline to stdout never fails");
}

#[cfg(test)]
mod tests {
    use super::write_json;

    #[test]
    fn write_json_emits_compact_object_and_trailing_newline() {
        let mut buf = Vec::new();
        write_json(&mut buf, &serde_json::json!({ "ok": true }));
        assert_eq!(buf, br#"{"ok":true}
"#);
    }
}
