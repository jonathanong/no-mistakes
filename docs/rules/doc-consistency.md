# `doc-consistency`

## Why and when

Use this rule when a repository needs a declarative documentation contract that
cannot be expressed by link reachability alone. Literal text checks keep
required phrases present and stale phrases absent. Scoped regex bans keep
pinned values, such as release pins, out of living documents and name the file
that should own them.

## What it catches

It requires configured files, headings, and substrings, and rejects configured
stale substrings in selected documentation. Each `bannedPatterns` entry rejects
non-empty regex matches on the files its globs select.

## Options

`requiredFiles`, `requiredSubstrings`, `bannedSubstrings`, and `bannedPatterns`
default to empty lists. `requiredHeading` defaults to unset. Each
`requiredSubstrings` item names one `file` and one literal `substring`. An
empty or omitted `bannedPatterns` list does no pattern work.

Each `bannedPatterns` entry requires `files`, `pattern`, and `message`. There
is no default message. `files` lists at least one non-empty glob. `pattern` is
a Rust regex and is not trimmed. `message` is reported verbatim after trimming
and must not be empty. Invalid globs, empty fields, and invalid regexes fail
configuration before any source is read, including when no file is selected.
The first invalid entry in index order fails the application, so a later valid
pattern produces no findings.

Search uses the leftmost match, then the next non-overlapping match. A
zero-width match is not a finding. A leading empty alternative can hide another
alternative at that same position. Inline flags such as `(?m)` and `(?s)` are
part of the pattern. The finding line is the count of newline bytes before the
match start, so a carriage return plus line feed is one line break and the
count is Unicode-safe. One finding is reported per pattern per start line.
Repeated matches of one pattern on that line collapse. Different patterns on
the same line each produce a finding. Identical messages on the same file and
line collapse.

Patterns run only on the intersection of the rule's existing file scope and the
entry globs. Literal `bannedSubstrings` and `requiredFiles` behavior is
unchanged. Substring findings stay on line 1.

## Valid example

A matching document contains every required heading and text, none of the
banned text, and no configured pin.

```yaml
rules:
  - rule: doc-consistency
    scope: repository
    options:
      requiredFiles: [README.md]
      requiredHeading: "## Install"
      requiredSubstrings:
        - file: README.md
          substring: "Install the CLI"
      bannedSubstrings: ["legacy API"]
      bannedPatterns:
        - files: ["docs/**/*.md"]
          pattern: 'PIN-[0-9]{3}'
          message: "Move this pin to versions.md."
```

## Counterexample

A README that omits the install section, a living doc that says `legacy API`,
or `docs/living.md` containing `PIN-777` fails. `PIN-999` in a file outside
`docs/**/*.md` does not match that glob. `(?m)^` and other zero-width matches
do not fail. `a*|QQ` against `QQ` can miss `QQ` because the empty alternative
matches first; `QQ|a*` reports `QQ`.

Fix: add the required file, heading, or substring, and remove the banned text.
For a regex hit, delete the match or move it to the owner file named in
`message`, and narrow `files` so the pattern does not cover that owner. Do not
suppress a pin that still belongs in the living document.

## Suppression

Use a line directive for one text finding, a next-line directive for the
following line, and a file directive only for generated documentation. Markdown
accepts the same directives inside an HTML comment. An apostrophe in the
prose, on an earlier line or on the same line, does not cancel that HTML line
directive. `//`, `#`, and `--` comments still work.

```markdown
<!-- no-mistakes-disable-file doc-consistency: generated catalog -->

PIN-102 <!-- no-mistakes-disable-line doc-consistency: tracked in versions.md -->

<!-- no-mistakes-disable-next-line doc-consistency: owner is versions.md -->
```

Prefer narrowing the file, substring, or pattern so future documentation stays
checked.

## Related rules

[`required-doc-section`](required-doc-section.md) is the narrower heading-only
policy. [`markdown-reachability`](markdown-reachability.md) validates discovery.
UUID bans stay on [`forbidden-calls`](forbidden-calls.md) through a
`moduleExport` target. Parent and child links stay on
[`markdown-child-links`](markdown-child-links.md), including `requireWholeFile`
and `countCanonicalHtmlListItems`.
