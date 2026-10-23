# `github-actions-download-integrity`

## Why and when

Use this rule when a workflow or composite action downloads bytes with `curl`
or `wget` inside a `run` script. An http(s) URL can change between runs, so a
later step can execute or unpack different bytes than the author read. The
rule is opt-in. Listing it is what turns it on. Product defaults do not enable
it, and it is not part of a recommended or strict preset.

At the default `max` effort, the rule reports only a download it can judge.
A URL it cannot resolve is not a finding. That silence means the URL was not
followed, not that the download is safe. Low effort uses the explicit
[unanalyzable policy](#effort) for supported downloader calls with indirect URLs.

## What it catches

The rule scans step `run` bodies in `.github/workflows/*.{yml,yaml}` (a file
in a subdirectory is not a workflow GitHub runs) and `runs.steps` in composite
actions (`.github/actions/**/action.{yml,yaml}` whose `runs.using` is
`composite`). Other action kinds are ignored. `defaults.run` is not a step
script.

It flags `curl`, `wget`, or a configured helper when an argument is an
`http://` or `https://` URL. `sudo`, `env`, `xargs`, and a shell prefix
(`if`, `!`, `(`, `{`, `time`, `exec`) do not hide that command, and
`if ! sha256sum -c` still counts as a check. A finding is skipped when any
of these is true:

- the same `run` body runs `sha256sum -c` (or `--check`, including a short
  cluster such as `-cw`) or `shasum -a 256 -c` (also `-a256` and `--check`).
  A command substitution or backtick in that check does not count
- the host, after stripping a port and any login prefix, is trusted
- the URL path is a checksum file (`.sha256`, `.sha256sum`, `.sha512`,
  `.sha512sum`, `SHA256SUMS`, `SHA256SUMS.txt`, `SHA512SUMS`, or
  `SHA512SUMS.txt`)
- the step carries `# integrity-check: skip reason=<slug>`

`<slug>` starts with a letter or digit and then uses letters, digits, `_`, or
`-`. The comment may be a shell comment in the `run` body, a trailing YAML
comment on a `run` line, or a YAML comment on the line immediately above the
step item.

A whole-line uppercase assignment is expanded only when that script has exactly
one such assignment and the value is a literal (`BASE_URL="https://example.invalid/base"`,
single quotes, or a bare token). `$NAME` and `${NAME}` are then replaced
outside single quotes. A second assignment, a lowercase name, `export`, a
value that contains `$`, or a command substitution is not expanded and is
not judged. A here-document body is skipped, and a download after its
terminator is still judged. An unterminated here-document is not judged.
`<<<` and `<<` inside `$((...))` are not here-documents. The rule does not
eval the shell.

A host that is still `$`, `` ` ``, or `${{ }}` after that expansion is not
judged. A literal untrusted host is judged even when the path still contains
`$`, because the path is then not a proven checksum file. Flow-style steps the
line scan cannot place are reported on line 1. Invalid YAML is not a finding,
which is not a pass. Rewrite anchors, aliases, and flow style into ordinary
block-style `run` bodies when a finding lands on line 1 and the download is
not on that line.

Compact `steps:` lists whose `-` item shares the `steps:` column are located
with the same step scanner as `github-actions-script-expressions`, so
`no-mistakes-disable-next-line` on the previous line matches the download.

## Options

`trustedHosts` is a list of host names. Ports are stripped before comparison,
and comparison is case-insensitive. Subdomains are different hosts. When the
rule is enabled and `trustedHosts` is omitted, the default is:

```text
localhost
127.0.0.1
api.github.com
objects.githubusercontent.com
```

An explicit list replaces the default. `trustedHosts: []` trusts nothing, so
`api.github.com` is flagged. Write IPv6 hosts in brackets (`[::1]`).

`downloadHelpers` adds command names that count as downloads when they take an
http(s) URL. The default is only `curl` and `wget`. An explicit list is extra,
not a replacement, so `[]` leaves `curl` and `wget` in place. A helper is a
single command token such as `ci_download_to`.

## Effort

`effort: max` is the default. It keeps the current expansion of one whole-line
uppercase assignment with a literal value, such as `BASE_URL="https://example.invalid/base"`,
before checking later `$BASE_URL` or `${BASE_URL}` URL arguments.

Select low effort and its explicit uncertainty policy on the rule application:

```yaml
rules:
  - rule: github-actions-download-integrity
    scope: repository
    options:
      effort: low
      unanalyzable: report
```

At `effort: low`, the rule does not expand assignments to URL values. If a
known downloader command has any dynamic candidate URL argument, the whole
command is unanalyzable: it emits one diagnostic and no definite unsafe-URL
findings from that command. The shared `unanalyzable` policy controls the
result: `report` fails, `advise` emits a non-failing advisory, and `ignore` skips
it. The static rewrite is to put the complete URL directly in the downloader's
URL argument. An unanalyzable URL does not prove that the download is unsafe;
`ignore` therefore leaves that command unjudged.

Low effort may inspect one uppercase literal binding to identify a downloader
or wrapper used as a dynamic command prefix. It does not use that binding to
expand URL values. When this proves a configured downloader behind an indirect
prefix, the shared `unanalyzable` policy applies; the rewrite is to call `curl`,
`wget`, or a configured helper directly. Commands that cannot be identified as
configured downloaders remain outside this rule's target scope. Dynamic
operands of known output and header options
keep using the existing option parser, so they do not trigger this policy as
candidate URL arguments.

Commands whose URL arguments are all literal keep the same checks at low and
max: trusted-host and checksum-file exceptions, same-step checksum verification,
and the `integrity-check: skip` comment still apply. URL-option parsing also
stays enabled, so header and output-file arguments are not treated as dynamic
URL slots.
The default `max` output is unchanged.

```yaml
rules:
  - rule: github-actions-download-integrity
    scope: repository
```

An explicit `trustedHosts` list replaces the default. `downloadHelpers` is extra
on top of `curl` and `wget`:

```yaml
rules:
  - rule: github-actions-download-integrity
    scope: repository
    options:
      trustedHosts:
        - example.invalid
      downloadHelpers:
        - ci_download_to
```

## Valid example

Verify the bytes in the same step, or download a checksum file from an
untrusted host. Trusted hosts may stay unverified.

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: |
          curl -fsSL https://example.invalid/tool.tgz -o tool.tgz
          curl -fsSL https://example.invalid/tool.tgz.sha256 -o tool.tgz.sha256
          sha256sum -c tool.tgz.sha256
      - run: curl https://api.github.com/repos/example/tool
      - run: |
          # integrity-check: skip reason=vendor-tarball
          curl -fsSL https://example.invalid/vendor.tgz -o vendor.tgz
```

A composite action uses the same `run` shape:

```yaml
runs:
  using: composite
  steps:
    - shell: bash
      run: |
        curl -fsSL https://example.invalid/tool.tgz -o tool.tgz
        sha256sum -c tool.sha256
```

## Counterexample

This step downloads an untrusted tarball and never checks the bytes:

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: curl -fsSL https://example.invalid/tool.tgz -o tool.tgz
```

## Fix

Pin the download in the same step:

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: |
          curl -fsSL https://example.invalid/tool.tgz -o tool.tgz
          sha256sum -c tool.tgz.sha256
```

`wget`, `shasum -a 256 -c`, and a configured helper use the same fix. A single
literal `BASE_URL="https://example.invalid/base"` followed by `curl "$BASE_URL/tool.tgz"`
is the same download. Two assignments, or `BASE_URL="$(...)"`, are not expanded.

## Suppression

Use a YAML comment. `no-mistakes-disable-next-line github-actions-download-integrity`
suppresses the next line. `no-mistakes-disable-line github-actions-download-integrity`
suppresses the same line. `no-mistakes-disable-file github-actions-download-integrity`
in the leading comment block suppresses the file.

```yaml
# no-mistakes-disable-next-line github-actions-download-integrity
- run: curl -fsSL https://example.invalid/tool.tgz -o tool.tgz
- run: curl -fsSL https://example.invalid/other.tgz -o other.tgz # no-mistakes-disable-line github-actions-download-integrity
```

`# integrity-check: skip reason=<slug>` is this rule's own skip for a download
that should not be hashed. It is not a `no-mistakes-disable` comment.

## Related rules

[`github-actions-pinned-hash`](github-actions-pinned-hash.md) pins action code.
[`github-actions-script-expressions`](github-actions-script-expressions.md)
rejects untrusted expressions inside scripts.
[`github-actions-checkout-credentials`](github-actions-checkout-credentials.md)
rejects a persisted write token. This rule only judges http(s) downloads in
`run` bodies.
