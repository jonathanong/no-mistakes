use super::valid_static_reference;

#[test]
fn static_image_references_follow_docker_name_tag_and_digest_shapes() {
    for image in [
        "node",
        "node:22",
        "library/node:22-alpine",
        "ghcr.io/example/service:v1.2.3",
        "localhost:5000/example/service",
        "example/service@sha256:abcdef0123456789abcdef0123456789",
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
    ] {
        assert!(!valid_static_reference(image), "{image}");
    }
}
