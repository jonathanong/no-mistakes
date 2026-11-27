use super::write_all;
use std::io;

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/invocation/deadline-json/unicode.json"
    )))
    .unwrap()
}

#[test]
fn unicode_and_short_writes_preserve_the_saved_utf16_units() {
    let data = fixture();
    let text = data["text"].as_str().unwrap();
    let expected: Vec<u16> = serde_json::from_value(data["utf16"].clone()).unwrap();
    for limit in [1, usize::MAX] {
        let mut actual = Vec::new();
        write_all(text, |units| {
            let count = units.len().min(limit);
            actual.extend_from_slice(&units[..count]);
            Ok(count)
        })
        .unwrap();
        assert_eq!(actual, expected);
    }
}

#[test]
fn large_chunks_do_not_split_supplementary_characters() {
    let data = fixture();
    // Put the first surrogate pair across the default chunk boundary.
    let text = format!(
        "{}{}",
        "a".repeat(8191),
        data["text"].as_str().unwrap().chars().last().unwrap()
    );
    let mut lengths = Vec::new();
    write_all(&text, |units| {
        lengths.push(units.len());
        assert!(!(0xd800..=0xdbff).contains(units.last().unwrap()));
        Ok(units.len())
    })
    .unwrap();
    assert_eq!(lengths, [8191, 2]);
}

#[test]
fn empty_output_does_not_call_the_console_writer() {
    write_all("", |_| panic!("empty output must not write")).unwrap();
}

#[test]
fn failed_zero_and_invalid_counts_fail_without_retrying_forever() {
    let data = fixture();
    let text = data["text"].as_str().unwrap();
    assert_eq!(
        write_all(text, |_| Err(io::ErrorKind::BrokenPipe.into()))
            .unwrap_err()
            .kind(),
        io::ErrorKind::BrokenPipe
    );
    assert_eq!(
        write_all(text, |_| Ok(0)).unwrap_err().kind(),
        io::ErrorKind::WriteZero
    );
    assert_eq!(
        write_all(text, |units| Ok(units.len() + 1))
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
}
