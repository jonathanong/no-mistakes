use super::{is_synthetic as judge, only_zero_versions};

fn is_synthetic(pin: &str) -> bool {
    judge(pin, pin)
}

#[test]
fn all_zero_versions_are_synthetic() {
    for pin in [
        "lychee-v0.0.0-test-x86_64-unknown-linux-gnu.tar.gz",
        "releases/download/0.0.0-test",
        "no-mistakes-v00.000.0-x.tar.gz",
        "v0.0",
    ] {
        assert!(is_synthetic(pin), "{pin}");
    }
}

#[test]
fn real_or_ambiguous_versions_are_not_synthetic() {
    for pin in [
        "lychee-v0.24.1-x86_64-unknown-linux-gnu.tar.gz",
        "actions/checkout@v4",
        "0.0.1",
        "1.2.3",
        "v0.0.0-test-1.2.3",
        "node:24",
        "x-0.",
        "x-0..0",
        "no digits at all",
        "",
    ] {
        assert!(!is_synthetic(pin), "{pin}");
    }
}

#[test]
fn reserved_registries_are_synthetic() {
    for pin in [
        "registry.test/example/cache:1.2.3",
        "registry.example/app:1.2.3",
        "registry.invalid/app:1.2.3",
        "app.localhost/app:1.2.3",
        "localhost:5000/app:1.2.3",
        "localhost/app:1.2.3",
        "example.com/app:1.2.3",
        "example.net/app:1.2.3",
        "example.org:5000/app:1.2.3",
        "registry.example.com/app:1.2.3",
        // A hostname is case-insensitive.
        "REGISTRY.TEST/app:1.2.3",
        "Example.COM/app:1.2.3",
        "LOCALHOST:5000/app:1.2.3",
        "Registry.Example.Com/app:1.2.3",
    ] {
        assert!(is_synthetic(pin), "{pin}");
    }
    for pin in [
        "registry.io/app:1.2.3",
        "GHCR.IO/example/app:1.2.3",
        "NotExample.com/app:1.2.3",
        "notexample.com/app:1.2.3",
        "example.com.evil.io/app:1.2.3",
        "ghcr.io/example/app:1.2.3",
        "valkey/valkey-bundle:9.1.0",
        "app:1.2.3",
    ] {
        assert!(!is_synthetic(pin), "{pin}");
    }
}

#[test]
fn repeated_digests_are_synthetic_only_when_untagged() {
    let zeroed = format!("app@sha256:{}", "0".repeat(64));
    let cycled = format!("registry.io/app@sha256:{}", "0123456789abcdef".repeat(4));
    let period_32 = format!(
        "app@sha256:{}",
        "0123456789abcdef0123456789abcdef".repeat(2)
    );
    assert!(is_synthetic(&zeroed));
    assert!(is_synthetic(&cycled));
    assert!(is_synthetic(&period_32));

    let real = "06ada57c26aa5cf429e9f2c0a99e3e4a42daecd45fc4c955d7c1399ab4227ae8";
    assert!(!is_synthetic(&format!("app@sha256:{real}")));
    assert!(!is_synthetic(&format!("app@sha256:{}", "0".repeat(63))));
    assert!(!is_synthetic(&format!("app@sha256:{}", "0".repeat(65))));
    assert!(!is_synthetic("app@sha256:"));
    assert!(!is_synthetic(&format!(
        "pgvector/pgvector:pg18@sha256:{}",
        "0".repeat(64)
    )));
}

#[test]
fn the_zero_rule_reads_only_the_versions_it_is_given() {
    // An action ref is judged whole for hosts but its zero versions are read
    // in a window, so a zero-version directory outside it is not a placeholder.
    let pin = "owner/repo/v0.0.0/path/action@v1";
    assert!(only_zero_versions(pin));
    assert!(is_synthetic(pin));
    assert!(!judge(pin, "path/action@v1"));
    // The reserved-host rule still sees the whole pin.
    assert!(judge("registry.test/a/b@v1", "a/b@v1"));
    // A placeholder inside the window is still one.
    assert!(judge("a/b/c@v0.0.0", "b/c@v0.0.0"));
}
