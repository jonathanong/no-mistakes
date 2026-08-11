use super::valid_static_reference;

#[test]
fn static_image_references_follow_docker_name_tag_and_digest_shapes() {
    for image in [
        "node",
        "node:22",
        "library/node:22-alpine",
        "ghcr.io/example/service:v1.2.3",
        "localhost:5000/example/service",
        "example/service@sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
    ] {
        assert!(valid_static_reference(image), "{image}");
    }
    for image in [
        "",
        "node:",
        "node@",
        "node@:sha256:abcdef0123456789abcdef0123456789",
        "node@sha256:",
        "Node:22",
        "node :22",
        "node::22",
        "ghcr.io//service:latest",
        "ghcr.io/example/service:bad tag",
        "-invalid/name:latest",
        "[not-an-ipv6/service",
        "[not-an-ipv6]:5000/service",
        "[::1]not-a-port/service",
    ] {
        assert!(!valid_static_reference(image), "{image}");
    }
}

#[test]
fn recognized_digest_algorithms_require_their_standard_hex_lengths() {
    for (algorithm, length) in [
        ("md5", 32),
        ("sha1", 40),
        ("sha224", 56),
        ("sha256", 64),
        ("sha384", 96),
        ("sha512", 128),
    ] {
        let valid = format!("node@{algorithm}:{}", "a".repeat(length));
        assert!(valid_static_reference(&valid), "{valid}");
        let short = format!("node@{algorithm}:{}", "a".repeat(length - 1));
        assert!(!valid_static_reference(&short), "{short}");
        let long = format!("node@{algorithm}:{}", "a".repeat(length + 1));
        assert!(!valid_static_reference(&long), "{long}");
    }
    let extensible = format!("node@vendor+v1:{}", "a".repeat(32));
    assert!(valid_static_reference(&extensible));
}
