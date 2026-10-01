use super::super::patterns::DEFAULT_PATTERNS;
use super::super::{check_source, compile_options, Options};
use super::RUNTIME_PATTERNS;
use regex::Regex;

mod homebrew_tests;

fn pins(line: &str) -> Vec<String> {
    let options = compile_options(&Options::default()).unwrap();
    check_source("src/app.test.mts", line, &options)
        .into_iter()
        .filter_map(|finding| finding.import)
        .collect()
}

fn assert_pins(line: &str, expected: &[&str]) {
    assert_eq!(pins(line), expected, "{line}");
}

#[test]
fn line_patterns_cannot_match_the_empty_string() {
    // The scanner resumes at the end of each pin, so an empty match would loop.
    for (_, regex, _) in RUNTIME_PATTERNS {
        assert!(!Regex::new(regex).unwrap().is_match(""), "{regex}");
    }
    for (_, regex, multiline) in DEFAULT_PATTERNS {
        let regex = regex.strip_prefix("(?<!@)").unwrap_or(regex);
        assert!(
            *multiline || !Regex::new(regex).unwrap().is_match(""),
            "{regex}"
        );
    }
}

#[test]
fn container_image_tags_are_pins() {
    for (line, expected) in [
        (
            "image: valkey/valkey-bundle:9.1.0",
            "valkey/valkey-bundle:9.1.0",
        ),
        ("image: valkey/valkey-bundle:9", "valkey/valkey-bundle:9"),
        (
            "\"image\": \"valkey/valkey-bundle:9.1.0\"",
            "valkey/valkey-bundle:9.1.0",
        ),
        ("'image: postgres:18'", "postgres:18"),
        ("image: redis:7.4-alpine", "redis:7.4-alpine"),
        ("image: node:24", "node:24"),
        ("image: 'a/b:1.2.3'", "a/b:1.2.3"),
        (
            "image: registry.gitlab.com:5050/g/p:9",
            "registry.gitlab.com:5050/g/p:9",
        ),
        ("FROM postgres:18", "postgres:18"),
        ("FROM \"postgres:18\"", "postgres:18"),
        (
            "FROM --platform=$BUILDPLATFORM golang:1.26 AS build",
            "golang:1.26",
        ),
        (
            "docker pull ghcr.io/acme/app:1.2.3.",
            "ghcr.io/acme/app:1.2.3",
        ),
        (
            "`run quay.io/org/tool:v1.2.3-rc.1 --help`",
            "quay.io/org/tool:v1.2.3-rc.1",
        ),
        (
            "mcr.microsoft.com/dotnet/sdk:9.0",
            "mcr.microsoft.com/dotnet/sdk:9.0",
        ),
        ("owner/name:v1.2.3", "owner/name:v1.2.3"),
        // Docker path components may contain `.`, `__`, and runs of `-`.
        ("owner/my.image:1.2.3", "owner/my.image:1.2.3"),
        ("image: owner/my.image:1.2.3", "owner/my.image:1.2.3"),
        ("owner/my__image--x:1.2.3", "owner/my__image--x:1.2.3"),
        (
            "image: owner/my__image--x:1.2.3",
            "owner/my__image--x:1.2.3",
        ),
        (
            "ghcr.io/acme/team/my.image:24-slim",
            "ghcr.io/acme/team/my.image:24-slim",
        ),
        ("image: owner/my.image:9", "owner/my.image:9"),
        ("pgvector/pgvector:pg18", "pgvector/pgvector:pg18"),
        ("pgvector/pgvector:pg18.1", "pgvector/pgvector:pg18.1"),
        ("otel/collector:0.153.0", "otel/collector:0.153.0"),
        ("library/node:24-slim", "library/node:24-slim"),
        ("ghcr.io/acme/api:5000", "ghcr.io/acme/api:5000"),
        (
            "registry.internal:5000/team/api:5000",
            "registry.internal:5000/team/api:5000",
        ),
    ] {
        assert_pins(line, &[expected]);
    }
}

#[test]
fn a_major_only_v_tag_is_an_image_after_image_or_from() {
    for (line, expected) in [
        ("image: repo:v2", "repo:v2"),
        ("image: owner/repo:v2", "owner/repo:v2"),
        ("\"image\": \"owner/repo:v2\"", "owner/repo:v2"),
        ("image: ghcr.io/acme/api:v2", "ghcr.io/acme/api:v2"),
        (
            "image: registry.io:5000/acme/api:v2",
            "registry.io:5000/acme/api:v2",
        ),
        ("FROM node:v2 AS build", "node:v2"),
        ("image: owner/repo:v2@sha256:${digest}", "owner/repo:v2"),
        // A dotted tag is still reported once, by the context-free pattern.
        ("image: owner/repo:v2.1", "owner/repo:v2.1"),
        ("image: repo:v2.1", "repo:v2.1"),
        ("image: ghcr.io/acme/api:v2.1", "ghcr.io/acme/api:v2.1"),
        ("image: owner/repo:9", "owner/repo:9"),
        ("image: ghcr.io/acme/api:9", "ghcr.io/acme/api:9"),
    ] {
        assert_pins(line, &[expected]);
    }
    for line in [
        "image: repo:v2beta",
        "image: owner/repo:v2beta",
        "image: ghcr.io/acme/api:v2beta",
        "image: repo:vNext",
    ] {
        assert!(pins(line).is_empty(), "{line}: {:?}", pins(line));
    }
}

#[test]
fn a_registry_host_v_tag_is_an_image_without_context() {
    let digest = "06ada57c26aa5cf429e9f2c0a99e3e4a42daecd45fc4c955d7c1399ab4227ae8";
    for (line, expected) in [
        ("ghcr.io/acme/api:v2", "ghcr.io/acme/api:v2"),
        (
            "expect(image).toBe('ghcr.io/acme/api:v2')",
            "ghcr.io/acme/api:v2",
        ),
        ("docker pull ghcr.io/acme/app:v2.", "ghcr.io/acme/app:v2"),
        (
            "registry.io:5000/acme/api:v2",
            "registry.io:5000/acme/api:v2",
        ),
        (
            "ghcr.io/acme/team/my.image:v2",
            "ghcr.io/acme/team/my.image:v2",
        ),
        ("ghcr.io/acme/api:v10", "ghcr.io/acme/api:v10"),
        // An interpolated digest leaves only the tag; a complete one is part of the pin.
        (
            "ghcr.io/acme/api:v2@sha256:${digest}",
            "ghcr.io/acme/api:v2",
        ),
        (
            &format!("ghcr.io/acme/api:v2@sha256:{digest}"),
            &format!("ghcr.io/acme/api:v2@sha256:{digest}"),
        ),
    ] {
        assert_pins(line, &[expected]);
    }
    // Only a registry host drops the context: the same shape on a slash-only
    // path stays an API or route key (`users/list:v2`) unless `image:` says otherwise.
    assert_pins(
        "owner/repo:v2 ghcr.io/acme/api:v2",
        &["ghcr.io/acme/api:v2"],
    );
    assert_pins(
        "image: ghcr.io/acme/api:v2 # ghcr.io/acme/api:v3",
        &["ghcr.io/acme/api:v2", "ghcr.io/acme/api:v3"],
    );
}

#[test]
fn container_image_digests_are_pins() {
    let digest = "06ada57c26aa5cf429e9f2c0a99e3e4a42daecd45fc4c955d7c1399ab4227ae8";
    for image in [
        "docker.io/library/redis",
        "redis",
        "ghcr.io/acme/app",
        "ghcr.io/acme/my.image",
        "acme/my__image--x",
    ] {
        let pin = format!("{image}@sha256:{digest}");
        assert_pins(&format!("expect(image).toBe('{pin}')"), &[&pin]);
    }
    let tagged = format!("pgvector/pgvector:pg18@sha256:{digest}");
    assert_pins(&tagged, &[&tagged]);
}

#[test]
fn a_tag_with_an_interpolated_or_short_digest_reports_only_the_tag() {
    // The digest is not concrete, but the tag in front of it is.
    for (line, expected) in [
        (
            "image: app/service:1.2.3@sha256:${imageDigest}",
            "app/service:1.2.3",
        ),
        (
            "mirror.gcr.io/library/node:26-trixie-slim@sha256:${imageDigest}",
            "mirror.gcr.io/library/node:26-trixie-slim",
        ),
        (
            "image: pgvector/pgvector:pg18@sha256:",
            "pgvector/pgvector:pg18",
        ),
        ("image: node:24@sha256:abc123", "node:24"),
        (
            "registry.io/team/api:5000@sha256:${digest}",
            "registry.io/team/api:5000",
        ),
    ] {
        assert_pins(line, &[expected]);
    }
    // A digest longer than 64 hex characters is not a digest either.
    let overlong = format!("app/service:1.2.3@sha256:{}", "a".repeat(65));
    assert_pins(&overlong, &["app/service:1.2.3"]);
}

#[test]
fn several_pins_on_one_line_are_all_reported() {
    assert_pins(
        "['valkey/valkey-bundle:9.1.0', 'otel/collector:0.153.0']",
        &["valkey/valkey-bundle:9.1.0", "otel/collector:0.153.0"],
    );
    assert_pins(
        "runs-on: [ubuntu-24.04, macos-15]",
        &["ubuntu-24.04", "macos-15"],
    );
    assert_pins(
        "brew link postgresql@17 openssl@3.5",
        &["postgresql@17", "openssl@3.5"],
    );
}

#[test]
fn setup_versions_are_pins_in_every_quoting_style() {
    for (line, expected) in [
        ("node-version: '26'", "node-version: '26'"),
        ("node-version: \"26\"", "node-version: \"26\""),
        ("node-version: 26", "node-version: 26"),
        ("python-version: 3.13'", "python-version: 3.13"),
        ("'node-version: \\'26\\''", "node-version: \\'26\\'"),
        ("\"node-version\": \"26\"", "node-version\": \"26\""),
        ("node-version: 26.x", "node-version: 26.x"),
        ("with: { java-version: 21 }", "java-version: 21"),
        ("go-version: 1.26.1\\n", "go-version: 1.26.1"),
    ] {
        assert_pins(line, &[expected]);
    }
}

#[test]
fn runner_labels_are_pins() {
    for (line, expected) in [
        ("runs-on: ubuntu-24.04-arm", "ubuntu-24.04-arm"),
        ("runs-on: ubuntu-22.04", "ubuntu-22.04"),
        ("os: 'macos-15'", "macos-15"),
        ("os: macos-15-intel", "macos-15-intel"),
        ("os: macos-14-large", "macos-14-large"),
        ("os: windows-2022", "windows-2022"),
        ("os: windows-2025-vs2026", "windows-2025-vs2026"),
        ("os: windows-11-arm", "windows-11-arm"),
        (
            "it('restricts ubuntu-24.04-arm to builds', run)",
            "ubuntu-24.04-arm",
        ),
        ("uses ubuntu-24.04.", "ubuntu-24.04"),
    ] {
        assert_pins(line, &[expected]);
    }
}

#[test]
fn lookalikes_are_not_pins() {
    for line in [
        "https://example.com/foo:8080/bar",
        "foo/bar:baz",
        // Known recall gap: outside `image:`/`FROM`, a bare `v2` is not version-shaped
        // and `owner/name:v2` reads like an API or route key.
        "owner/name:v2",
        "users/list:v2 api/keys:v2",
        // Dotted path components must not turn filenames with a line number into images.
        "src/a.b.mts:12",
        "at x (src/a.b.mts:12:5)",
        "cdn.jsdelivr.net/npm/pkg/dist/index.min.js:1",
        "https://github.com/org/repo/blob/main/src/a.b.ts:12",
        "see docs/a.b.md:12 and lib/x.test.mts:3:9",
        // Not Docker separators: `..`, `___`, and a leading `-`.
        "owner/my..image:1.2.3",
        "owner/my___image:1.2.3",
        "owner/-image:1.2.3",
        "at run (src/file.mts:12:5) at dev/initialize:2",
        "node:internal/modules/run_main:107",
        "github.com/acme/app/internal/binder.go:1755 +0x1a4",
        "docs/rules.md:12 127.0.0.1:5432 localhost:3000/health",
        "https://github.com/org/repo/blob/main/src/a.ts:12",
        // A registry-host `:v<N>` needs a clean boundary on both sides: a scheme
        // or `//` in front, a path, `-suffix`, or word after the tag are not images.
        "https://ghcr.io/acme/api:v2",
        "https://api.acme.io/users:v2/list",
        "http://10.0.0.5:8080/api:v2",
        "git+ssh://git.acme.io/team/repo:v2",
        "ghcr.io/acme/api:v2/path",
        "ghcr.io/acme/api:v2-beta",
        "ghcr.io/acme/api:v2beta ghcr.io/acme/api:vNext ghcr.io/acme/api:v",
        // Go module paths carry `/v2` as a path element, never as a `:` tag.
        "github.com/acme/app/v2",
        "go get github.com/acme/app/v2/pkg",
        "github.com/acme/app/v2/internal/binder.go:1755 +0x1a4",
        "github.com/acme/app/v2/internal/binder.go:12:5",
        // A host with a port, or a host followed by a bare tag, has no repository path.
        "api.acme.io:8080/v2",
        "api.acme.io:v2",
        "db.acme.io:5432 v2",
        "http://localhost:3000/a:1.2",
        "ssh://git@github.com:22/org/repo.git",
        "-p 51088:6379 valkey/valkey-bundle:",
        "12/25:1.5 16/9:1.5 a/b:1",
        "application/json:2 w-1/2:hover",
        "image: postgres",
        "image: latest",
        "image: valkey/valkey-bundle:latest",
        "FROM node AS build",
        "const image = `pgvector/pgvector:${tag}@sha256:${digest}`",
        "/^pgvector\\/pgvector:[\\w.-]+@sha256:[0-9a-f]{64}$/u",
        "docker.io/library/redis@sha256:short",
        "ubuntu-latest ubuntu-slim macos-latest windows-latest windows-1252",
        "ubuntu-24.04.iso docs/ubuntu-22.04-notes.md macos-15.png",
        "ubuntu-24.041 xubuntu-24.04 macos-150",
        "node-version-file: .nvmrc",
        "node-version: lts/*",
        "node-version: ${{ matrix.node }}",
        "python-version: \"${{ matrix.python }}\"",
        ".node-version: 26",
    ] {
        assert!(pins(line).is_empty(), "{line}: {:?}", pins(line));
    }
}

#[test]
fn placeholder_values_are_not_pins() {
    let zeroed = "0".repeat(64);
    let cycled = "0123456789abcdef".repeat(4);
    for line in [
        "lychee-v0.0.0-test-x86_64-unknown-linux-gnu.tar.gz",
        "https://example.test/releases/download/0.0.0-test/tool.tgz",
        "registry.test/example/cache:1.2.3",
        "registry.example.com/app:2.4.1",
        "localhost:5000/app:2.4.1",
        "registry.example.com/checkout:v1",
        "registry.test/app:v2",
        "localhost:5000/app:v2",
        &format!("app@sha256:{zeroed}"),
        &format!("registry.test/app@sha256:{cycled}"),
    ] {
        assert!(pins(line).is_empty(), "{line}: {:?}", pins(line));
    }
    assert_pins(
        "lychee-v0.24.1-x86_64-unknown-linux-gnu.tar.gz",
        &["lychee-v0.24.1-x86_64-unknown-linux-gnu.tar.gz"],
    );
    assert_pins("registry.io/app:1.2.3", &["registry.io/app:1.2.3"]);
}
