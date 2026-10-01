//! Default line patterns for pins in container images, CI setup inputs,
//! Homebrew formulae, and runner labels.
//!
//! Every pattern captures the pin in a named `pin` group and carries its own
//! left and right boundary so prose, stack frames, paths, and `file.ts:12`
//! locations are not read as versions. The scanner resumes each search at the
//! end of the pin, which lets one separator character close a pin and open the
//! next.
//!
//! Docker references are ASCII-only. Digits are `[0-9]` and word boundaries are
//! `(?-u:\b)`, never `\d`/`\b`, so a non-ASCII character always ends a reference
//! (`ghcr.io/acme/api:v2β` reports `ghcr.io/acme/api:v2`) and is never a digit.

/// `(reason, regex, line_context)`: the pattern only runs on lines matching
/// `line_context` when one is given.
pub(super) type RuntimePattern = (&'static str, &'static str, Option<&'static str>);

/// A pin may start at the line start, after a JavaScript `\n`/`\r`/`\t` escape,
/// or after a character that cannot continue a path, name, or version.
macro_rules! left {
    () => {
        r"(?:^|\\[nrt]|[^A-Za-z0-9_.:/@\\-])"
    };
}

/// A pin ends at the line end or at a character that cannot continue a tag.
/// A single trailing `.` (sentence end) is allowed.
macro_rules! right {
    () => {
        r"(?:$|[^A-Za-z0-9_.:/@+-]|\.(?:$|[^0-9A-Za-z_-]))"
    };
}

/// Repository path component in Docker's grammar: alphanumeric runs joined by
/// `.`, `_`, `__`, or `-`+, as in `my.image`. Used only where the tag is
/// version-shaped or `v<N>`, after `image:`/`FROM`, or on a digest, so
/// `a.b.mts:12` is never read as an image.
macro_rules! comp_dot {
    () => {
        r"[a-z0-9]+(?:(?:\.|_{1,2}|-+)[a-z0-9]+)*"
    };
}

/// Dotless component for the context-free bare-integer tag; a dot would let
/// `internal/binder.go:1755` and `page.tsx:1:1` through.
macro_rules! comp {
    () => {
        r"[a-z0-9]+(?:(?:_{1,2}|-+)[a-z0-9]+)*"
    };
}

/// First component: starts with a letter, so `51088:6379` is never an image.
macro_rules! comp_l {
    () => {
        r"[a-z][a-z0-9]*(?:(?:_{1,2}|-+)[a-z0-9]+)*"
    };
}

/// One DNS label (RFC 1123): alphanumeric at both ends, hyphens only inside.
macro_rules! label {
    () => {
        "[a-z0-9](?:[a-z0-9-]*[a-z0-9])?"
    };
}

/// Registry host: a dotted name (with optional port) or `localhost:port`.
macro_rules! host {
    () => {
        concat!(
            "(?:(?:",
            label!(),
            r"\.)+",
            label!(),
            "(?::[0-9]{1,5})?|localhost:[0-9]{1,5})"
        )
    };
}

/// Version-shaped tags: dotted numeric, `N-variant`, or `pgN`. Digits are
/// ASCII, never `\d`, so Unicode digits are not versions.
macro_rules! tag {
    () => {
        concat!(
            r"(?:v?[0-9]+(?:\.[0-9]+)+",
            sfx!(),
            r"|[0-9]+-[A-Za-z][A-Za-z0-9_]*(?:[.-][A-Za-z0-9_]+)*",
            r"|pg[0-9]+(?:\.[0-9]+)*",
            sfx!(),
            ")"
        )
    };
}

macro_rules! sfx {
    () => {
        r"(?:[-+][A-Za-z0-9_]+(?:[.-][A-Za-z0-9_]+)*)?"
    };
}

/// An optional trailing digest is part of the pin only when it is complete.
macro_rules! digest_opt {
    () => {
        r"(?:@sha256:[0-9a-f]{64})?"
    };
}

/// End of a tagged image: a normal pin end, or the `@sha256:` of an
/// interpolated or short digest, so `app:1.2.3@sha256:${digest}` reports only
/// the concrete tag instead of being dropped or reported with a stub digest.
macro_rules! image_end {
    () => {
        concat!("(?:", right!(), "|@sha256:)")
    };
}

/// `image:` values, `FROM` lines: the only places a slashless `postgres:18`,
/// a bare-integer `valkey/valkey-bundle:9`, or a major-only `repo:v2` is
/// unambiguously an image (`users/list:v2` is an API key elsewhere). A path
/// under a registry host needs no context for either `:9` or `:v2`.
macro_rules! image_context {
    () => {
        r#"(?:(?-u:\b)image\\?["']?:\s*(?:\\?["'])?|(?-u:\b)FROM\s+(?:--platform=\S+\s+)?["']?)"#
    };
}

pub(super) const RUNTIME_PATTERNS: &[RuntimePattern] = &[
    (
        "container image tag",
        concat!(
            left!(),
            r"(?P<pin>(?:",
            host!(),
            "/",
            comp!(),
            "(?:/",
            comp!(),
            r")*:[0-9]+|(?:",
            host!(),
            "/",
            comp_dot!(),
            "|",
            comp_l!(),
            "/",
            comp_dot!(),
            ")(?:/",
            comp_dot!(),
            ")*:",
            tag!(),
            "|",
            host!(),
            "/",
            comp_dot!(),
            "(?:/",
            comp_dot!(),
            r")*:v[0-9]+)",
            digest_opt!(),
            ")",
            image_end!()
        ),
        None,
    ),
    (
        "container image tag",
        concat!(
            image_context!(),
            "(?P<pin>(?:",
            comp_l!(),
            ":(?:",
            tag!(),
            r"|v?[0-9]+)|",
            comp_l!(),
            "(?:/",
            comp_dot!(),
            r")+:v?[0-9]+)",
            digest_opt!(),
            ")",
            image_end!()
        ),
        None,
    ),
    // `image:ghcr.io/acme/api:v2` with nothing between the key and the host: the
    // context-free pattern's left boundary rejects the `:`, so this tight form
    // keeps it. Any space or quote after the colon belongs to that pattern, so a
    // tag is never reported twice.
    (
        "container image tag",
        concat!(
            r#"(?-u:\b)image\\?["']?:"#,
            "(?P<pin>",
            host!(),
            "/",
            comp_dot!(),
            "(?:/",
            comp_dot!(),
            r")*:v[0-9]+",
            digest_opt!(),
            ")",
            image_end!()
        ),
        None,
    ),
    (
        "container image digest",
        concat!(
            left!(),
            "(?P<pin>(?:",
            host!(),
            "/)?",
            comp_dot!(),
            "(?:/",
            comp_dot!(),
            ")*@sha256:[0-9a-f]{64})",
            right!()
        ),
        None,
    ),
    (
        "setup action version",
        concat!(
            r"(?:^|[^A-Za-z0-9_.-])(?P<pin>(?:node|python|go|java|ruby|dotnet|php|bun|deno)",
            r#"-version(?:\\?["'])?:\s*(?:"#,
            r#"\\?"[0-9][A-Za-z0-9._+-]*\\?"|\\?'[0-9][A-Za-z0-9._+-]*\\?'|[0-9][A-Za-z0-9._+-]*))"#
        ),
        None,
    ),
    (
        "versioned Homebrew formula",
        r"(?:^|[^A-Za-z0-9_.@-])(?P<pin>[a-z][a-z0-9+_-]*@[0-9]+(?:\.[0-9]+)?)(?:$|[^A-Za-z0-9_.@-]|\.(?:$|[^0-9A-Za-z_-]))",
        Some(r"(?i)(?-u:\b)(?:brew|homebrew|linuxbrew)(?-u:\b)|/Cellar/"),
    ),
    (
        "versioned runner label",
        concat!(
            left!(),
            r"(?P<pin>ubuntu-[0-9]{2}\.[0-9]{2}(?:-arm)?|macos-[0-9]{2}(?:-(?:intel|large|xlarge))?",
            r"|windows-(?:20[0-9]{2}|11-arm)(?:-vs[0-9]{4})?)",
            r"(?:$|[^A-Za-z0-9_.-]|\.(?:$|[^0-9A-Za-z_-]))"
        ),
        None,
    ),
];

#[cfg(test)]
mod tests;
