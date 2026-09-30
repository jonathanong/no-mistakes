use super::super::patterns::DEFAULT_PATTERNS;
use super::super::{check_source, compile_options, Options};
use super::RUNTIME_PATTERNS;
use regex::Regex;

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
fn container_image_digests_are_pins() {
    let digest = "06ada57c26aa5cf429e9f2c0a99e3e4a42daecd45fc4c955d7c1399ab4227ae8";
    for image in ["docker.io/library/redis", "redis", "ghcr.io/acme/app"] {
        let pin = format!("{image}@sha256:{digest}");
        assert_pins(&format!("expect(image).toBe('{pin}')"), &[&pin]);
    }
    let tagged = format!("pgvector/pgvector:pg18@sha256:{digest}");
    assert_pins(&tagged, &[&tagged]);
    assert_pins(
        "image: pgvector/pgvector:pg18@sha256:",
        &["pgvector/pgvector:pg18@sha256:"],
    );
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
fn homebrew_formulae_need_homebrew_context() {
    for line in [
        "brew install postgresql@18",
        "brew services start postgresql@18",
        "brew install --cask temurin@21",
        "/opt/homebrew/opt/postgresql@18/bin",
        "/usr/local/Cellar/postgresql@18/18.1",
        "Homebrew formula postgresql@18.",
    ] {
        assert!(
            pins(line)
                .iter()
                .any(|pin| pin.starts_with("postgresql@") || pin.starts_with("temurin@")),
            "{line}"
        );
    }
    for line in [
        "undici@1.0.1",
        "pnpm@12",
        "packageManager: 'pnpm@12.0.0'",
        "brewery@2",
        "brew install postgresql",
        "brew services start postgresql@\\d+",
    ] {
        assert!(pins(line).is_empty(), "{line}");
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
        "at run (src/file.mts:12:5) at dev/initialize:2",
        "node:internal/modules/run_main:107",
        "github.com/acme/app/internal/binder.go:1755 +0x1a4",
        "docs/rules.md:12 127.0.0.1:5432 localhost:3000/health",
        "https://github.com/org/repo/blob/main/src/a.ts:12",
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
