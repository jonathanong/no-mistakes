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
a Rust regex and is not trimmed. It must not be able to match the empty string.
`a*`, `a?`, `(a|)?`, `(|b)`, `(?m)^`, and `a*|QQ` fail configuration. The error
echoes the pattern, says that empty-matching branch hides later alternatives,
and tells you to use `+` instead of `*` or drop the empty branch. A pattern
that cannot match empty, such as `a+` or `PIN-[0-9]{3}`, is unchanged. `message`
is reported verbatim after trimming and must not be empty. Invalid globs, empty
fields, empty-matching patterns, patterns over the explicit regex size limit,
and invalid regexes fail configuration before any source is read, including
when no file is selected. An oversized pattern such as `(a{1000}){1000}` reports
that the pattern exceeds the 10 MiB compiled-size limit and to reduce
repetition counts, not a generic invalid-regex error. The size limit is the
regex crate's 10 MiB ceiling, set explicitly. The
first invalid entry in index order fails the application, so a later valid
pattern produces no findings.

Search uses the leftmost match, then the next non-overlapping match. Inline
flags such as `(?m)` and `(?s)` are part of the pattern. The finding line is
the count of newline bytes before the match start, so a carriage return plus
line feed is one line break and the count is Unicode-safe. One finding is
reported per pattern per start line. Repeated matches of one pattern on that
line collapse. Different patterns on the same line each produce a finding.
Identical messages on the same file and line collapse.

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
`docs/**/*.md` does not match that glob. `a*|QQ`, `QQ|a*`, and `(?m)^` are
configuration errors because they can match the empty string. `a+` is not.

Fix: add the required file, heading, or substring, and remove the banned text.
For a regex hit, delete the match or move it to the owner file named in
`message`, and narrow `files` so the pattern does not cover that owner. Do not
suppress a pin that still belongs in the living document.

## Suppression

Use a line directive for one text finding, a next-line directive for the
following line, and a file directive only for generated documentation. `.md`
and `.markdown` accept the same directives inside an HTML comment. An
apostrophe in that prose, on an earlier line or on the same line, does not
cancel an HTML `no-mistakes-disable-line` or a leading HTML
`no-mistakes-disable-next-line`. `.mdx` keeps JavaScript quote tracking on
purpose, so an apostrophe there can hide a later HTML line directive. `//`,
`#`, and `--` comments still work.

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
