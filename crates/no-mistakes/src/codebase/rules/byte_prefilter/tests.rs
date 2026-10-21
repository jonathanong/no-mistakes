use super::*;

#[test]
fn hex_run_requires_consecutive_digits() {
    assert!(contains_hex_run(&[b'a'; 40], 40));
    assert!(!contains_hex_run(&[b'a'; 39], 40));
    let mut split = vec![b'a'; 39];
    split.push(b'_');
    split.extend([b'a'; 39]);
    assert!(!contains_hex_run(&split, 40));
    assert!(contains_hex_run(
        b"xx0123456789ABCDEFabcdef0123456789ABCDEF0123yy",
        40
    ));
}

#[test]
fn ascii_ignore_case_finds_mixed_case_needles() {
    assert!(contains_ascii_ignore_case(b"user@Example.COM", b"example"));
    assert!(contains_ascii_ignore_case(
        b"person%40EXAMPLE%2ecom",
        b"example"
    ));
    assert!(!contains_ascii_ignore_case(b"user@exampl.org", b"example"));
    assert!(contains_ascii_ignore_case(b"bind", b"bind"));
    assert!(!contains_ascii_ignore_case(b"bin", b"bind"));
    assert!(contains_any_ascii_ignore_case(
        b"Mail.Example.Com",
        &[b"example".to_vec().into_boxed_slice()]
    ));
    assert!(contains_bytes(b"server.listen(0)", b"listen"));
    assert!(!contains_bytes(b"server.bind((h, 0))", b"listen"));
    assert!(contains_ascii_ignore_case(b"", b""));
    assert!(is_ascii(b"user@okta.com"));
    assert!(!is_ascii("user@oKta.com".as_bytes()));
}
