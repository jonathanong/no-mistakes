# `doc-consistency`

## Why and when

Use this rule when a repository needs a declarative documentation contract that
cannot be expressed by link reachability alone.

## What it catches

It requires configured files, headings, and substrings, and rejects configured
stale substrings or configured regular expressions in selected documentation.

## Options

`requiredFiles`, `requiredSubstrings`, and `bannedSubstrings` default to empty
lists. `requiredHeading` defaults to unset. Each `requiredSubstrings` item names
one `file` and one literal `substring`. `bannedPatterns` also defaults to an empty
list; each entry requires nonempty `files`, `pattern`, and `message`. Leaving
every option at its default makes
the rule a no-op.

## Valid example

A matching document containing every required heading and text and none of the
banned text passes.

## Related rules

[`required-doc-section`](required-doc-section.md) is the narrower heading-only
policy; [`markdown-reachability`](markdown-reachability.md) validates discovery.

## Suppression

Use a line directive for a single text finding when it has a line location; use
a file directive only for generated documentation. Prefer narrowing the matching
file or substring policy so future documentation remains checked.

Requires documentation files, headings, substrings, and banned-substring
policies.

```yaml
rules:
  - rule: doc-consistency
    scope: repository
    options:
      requiredFiles: [README.md]
      requiredHeading: "## Install"
```

Counterexample: a README that omits the install section or points to stale docs.

Fix: add the required file/heading/substrings and remove banned text.

## Scoped regex bans

Use `bannedPatterns` when living documents should point at an owner file rather
than repeat changing values. Each pattern has explicit file globs and its own
message. The globs intersect the application's existing roots, include/exclude
filters and visible-file inventory; they do not discover additional files.

```yaml
rules:
  - rule: doc-consistency
    scope: repository
    options:
      requiredFiles: [README.md]
      bannedPatterns:
        - files: [README.md, "docs/**/*.md"]
          pattern: '(?m)^Current SHA:\s*[a-f0-9]{7,40}\b'
          message: "Keep live pins in tools/pins.json; link that owner file."
```

Counterexample: `Current SHA: abcdef1234567` in a selected document repeats a live
pin. Compliant example: `See tools/pins.json for the current source revision.`
Fix: remove the repeated value and link the owner named by the configured message.
No SHA, version or owner-file convention is enabled by default.

Patterns use Rust `regex` syntax: matching is case-sensitive unless the pattern
sets a flag such as `(?i)`, and `(?m)` enables line anchors. A multiline match
reports its starting line. Repeated matches of one pattern on one line produce
one finding; separate patterns can intentionally report separate messages.
Invalid regexes/globs or missing/empty required fields are configuration errors,
even when no files match, and cannot be suppressed. Patterns compile once per
application; source reads share the request's existing source store.

Regex findings support normal single-line HTML comments in Markdown. They honor `no-mistakes-disable-file`, `no-mistakes-disable-line`, and
`no-mistakes-disable-next-line`. For example:

```text
<!-- no-mistakes-disable-next-line doc-consistency: immutable historical release evidence -->
Current SHA: abcdef1234567
```

Existing `bannedSubstrings` remain literal checks with their existing messages.
The asynchronous Node `check()` and `analyzeProject()` check report consume the
same configuration and produce the same scoped findings and suppressions.

HTML directives require one closed comment ending the line. Multiple comments,
unclosed comments, or text after `-->` do not suppress findings. Existing `//`,
`#`, and SQL `--` directives retain their existing behavior.
