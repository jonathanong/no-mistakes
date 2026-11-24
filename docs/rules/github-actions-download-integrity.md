# `github-actions-download-integrity`

## Why and when

Use this rule when a workflow, composite action, or explicitly selected shell
source downloads bytes with `curl` or `wget`. An http(s) URL can change between
runs, so a later step can execute or unpack different bytes than the author read. The
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

- with `strictSequencing: false`, the same `run` body or selected shell source
  runs `sha256sum -c` (or `--check`, including a short
  cluster such as `-cw`) or `shasum -a 256 -c` (also `-a256` and `--check`).
  A command substitution or backtick in that check does not count
- the host, after stripping a port and any login prefix, is trusted
- the URL path is a checksum file (`.sha256`, `.sha256sum`, `.sha512`,
  `.sha512sum`, `SHA256SUMS`, `SHA256SUMS.txt`, `SHA512SUMS`, or
  `SHA512SUMS.txt`)
- a reasoned `# integrity-check: skip reason=<slug>` matches the selected annotation scope

`<slug>` starts with a letter or digit and then uses letters, digits, `_`, or
`-`. The comment may be a shell comment in the `run` body, a trailing YAML
comment on a `run` line, or a YAML comment on the line immediately above the
step item.

At max effort, each whole-line uppercase assignment is considered independently.
A name must be assigned exactly once in the step or selected script, before its
use. Literal values may use single quotes, double quotes, or a bare token.
`$NAME` and `${NAME}` are replaced outside single quotes. Multiple different
names no longer prevent expansion. Repeated assignments or observed writes,
including `read`, `export`, and `unset`, prevent recovery of that name; they do
not discard an unrelated stable binding. Commands that can write unknown names,
such as `eval` or sourcing another script, prevent a binding proof.
Lowercase names and declarations such as `export URL=...` are not recovered as
whole-line literal bindings.

A partly literal value can establish its URL host when the prefix before the
first `$` already contains a complete `http://host/` or `https://host/` authority
and path boundary. The dynamic path remains dynamic and cannot establish a
checksum-file exception:

```sh
BASE_URL="https://example.invalid/v${VERSION}"
FIRST_URL="https://example.invalid/first.tgz"
SECOND_URL="https://example.invalid/second.tgz"
curl "${BASE_URL}/tool.tgz"
curl "$FIRST_URL"
wget "$SECOND_URL"
```

All three downloads above are reported without verification. A checksum check
or a configured trusted host still exempts them. A dynamic scheme, host, or
port does not establish a host and is not judged at max effort:

```sh
BASE_URL="https://${HOST}/v1"
curl "${BASE_URL}/tool.tgz"
```

Shell word splitting and dynamic argument roles cannot establish a URL proof;
quote variable URL operands. Assignment-looking text inside comments or
here-document data is not a binding. Assignment substitution preserves
here-document delimiters and bodies. The scanner skips each body in opener
order and checks downloads after the last terminator. An unterminated here-document
is not judged. `<<<` and `<<` inside `$((...))` are not here-documents. The rule
does not eval the shell.

At max effort in Bourne-style steps, a literal HTTP(S) URL in the step’s `env`
map can also bind `"$NAME"` or `"${NAME}"` as a complete downloader URL argument
(including `--url="$NAME"`). An unquoted `$NAME` or `${NAME}` that is the whole
shell word binds the same way (`curl -o tool.tgz ${TOOL_URL}`,
`wget -O tool.tgz $TOOL_URL`). An unquoted value that contains whitespace stays
unresolved, because the shell word-splits it. An unquoted value that contains
`*`, `?`, or a `[` closed by a later `]` stays unresolved, because the shell
pathname-expands it. A lone `[`, or a `]` with no `[`, still binds. Those
glob characters inside a quoted expansion still bind. A composition such
as `$BASE/$FILE` or `${NAME}.tgz`, an unquoted `--url=$NAME`, and a command
whose option slots contain other unquoted expansions stay unresolved, because
word splitting could change which word is the URL. Explicit or inherited
unsupported shells do not use step-env resolution, nor does an implicit
shell on a known Windows runner.
Otherwise, an omitted effective `shell` uses the scanner’s Bourne-style
assumption. Unresolved runner expressions leave the effective shell unknown
and do not enable step-env resolution without an explicit supported shell.
Single-quoted shell variables remain literal. Values containing
GitHub expressions or shell expansion stay unresolved; job and workflow env
maps are not evaluated. A shell reassignment leaves the original step binding
unresolved. Plain literal output or header bindings can coexist with the URL
binding when the downloader’s URL argument role remains statically known.
Literal binding data is never allowed to create shell commands; values that
would introduce shell syntax remain unresolved.

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

`shellSourceInclude` selects additional Git-tracked shell sources with
root-relative path globs, intersected with the rule application’s file scope.
There is no extension filter: each matched tracked file is treated as shell
source, except recognized workflow and action YAML keeps its normal handling.
Without a Git index, no additional shell sources are scanned.
It defaults to `[]`, so scripts invoked by workflows or composite
actions are not followed automatically. Select only the scripts whose download
policy you intend to enforce; the scanner reads source and never executes it.
Selected scripts use the same host, checksum, helper, and suppression policy
as inline scripts, with findings on the script’s own lines. A reasoned
`# integrity-check: skip reason=<slug>` anywhere in a selected shell source
skips that source’s downloads. The `#` must begin a shell comment at a word
boundary; `echo note# integrity-check: skip reason=fixture` is an ordinary
argument and does not skip analysis.

`skipAnnotationScope` is `"step"` (the default) or `"line"`. The default preserves
the existing whole-step annotation scope for workflows and composite actions,
and whole-file scope for selected shell sources. With `"line"`, a trailing
annotation applies only to downloads on that physical line; a standalone
annotation applies only to the immediately following physical line. A blank
line breaks that association. Annotation-looking strings and here-document
payloads are not comments. An annotation on an inline YAML `run` stays within
that step; an adjacent literal block header can annotate its first body line.
A URL on a backslash continuation line can still produce a finding when the
skip applies to the command's first line. Put the URL on the final physical
line and append the annotation there. To annotate a URL on the first line,
put a standalone annotation immediately above the command.

For line scope, use ordinary inline runs or literal `run: |` bodies. Folded
`run: >` bodies and other ambiguous physical-line projections retain their
findings instead of applying an annotation that could suppress extra downloads. Rewrite them
as literal blocks when you need a line-specific exception.

```yaml
rules:
  - rule: github-actions-download-integrity
    scope: repository
    options:
      skipAnnotationScope: line
```

In line mode, the last download below remains a finding:

```sh
# integrity-check: skip reason=separate-review
curl https://example.invalid/first
curl https://example.invalid/second # integrity-check: skip reason=separate-review
curl https://example.invalid/third
```

`strictSequencing` defaults to `false`. The default accepts a supported checksum
command anywhere in the same script. When enabled, each untrusted payload must
have a supported check after it and before the next payload transfer, including
one from a trusted host. Each payload URL starts a new window, even within one
downloader command. A known checksum sidecar stays in the preceding payload’s
window; an unresolved dynamic download candidate closes it. Split multiple
payload downloads into separate commands with a check between them.

This is a lexical ordering check. It does not prove shell control flow or that
the checksum file describes the downloaded bytes.

```yaml
rules:
  - rule: github-actions-download-integrity
    scope: repository
    options:
      shellSourceInclude:
        - .github/actions/tool/install.sh
        - scripts/install-*.sh
      strictSequencing: true
```

With strict sequencing, verify each payload before fetching the next one:

```sh
curl -fsSL https://example.invalid/first -o first
sha256sum -c first.sha256
curl -fsSL https://example.invalid/second -o second
sha256sum -c second.sha256
```

Moving the first check below the second download reports the first download
under strict sequencing. With the option off, the same script retains the
same-script checksum policy.

## Analysis

Workflow and composite-action checks reuse parsed YAML for the same source text
within one check invocation. Shell script analysis reads its selected shell sources.

### Effort

- Supports `low` and `max` (default `max`). **Max**: resolves independently assigned uppercase bindings and partly literal URL values whose host is already fixed before a dynamic path. **Low**: accepts only literal URL arguments and routes a command with any dynamic candidate URL to `unanalyzable`. No partner rule is required.

`effort: max` is the default. It resolves eligible whole-line uppercase
assignments before checking later `$BASE_URL` or `${BASE_URL}` URL arguments,
including multiple distinct names and partly literal values with a fixed host.
This changes findings for URL assignments that previously stayed unresolved.

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
`ignore` therefore leaves that command without analysis.
With `report`, low can produce more findings than max when max resolves a
dynamic URL to a trusted host or another allowed form: this is the explicit
price of skipping expansion, rather than proof of an unsafe download.

A dynamic option name such as `curl --$FLAG URL` also makes argument slots
uncertain: the expanded option could consume the apparent URL as a filename.
It follows the same policy. Fixed option names with dynamic values, such as
`--output="$OUT"` or `--header="$HEADER"`, keep their known operand positions.
An unquoted dynamic value can split into additional options, so it also makes
the command unanalyzable. Quote those values to keep their argument positions.

Low effort may inspect one uppercase literal binding to identify a downloader
or wrapper used as a dynamic command prefix. It does not use that binding to
expand URL values. When this proves a configured downloader behind an indirect
prefix, the shared `unanalyzable` policy applies; the rewrite is to call `curl`,
`wget`, or a configured helper directly. Commands that cannot be identified as
configured download commands remain outside this rule's target scope. Quoted
dynamic operands of known output and header options
keep using the existing option parser, so they do not trigger this policy as
candidate URL arguments.
Even a literal trusted-host URL behind a proved indirect prefix follows the
policy: low does not use alias recovery to prove the complete argument shape.

Commands whose URL arguments are all literal keep the same checks at low and
max: trusted-host and checksum-file exceptions, same-step checksum verification,
and the `integrity-check: skip` comment still apply. URL-option parsing also
stays enabled, so header and output-file arguments are not treated as dynamic
URL slots.
Literal-only commands retain their existing host and verification policy.

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

### Unanalyzable inputs

Following [recall over precision](../../README.md#how-analysis-works), inputs this rule can't prove are never treated as clean; they follow `options.unanalyzable`.

- Inputs this rule cannot prove (a downloader URL built from a variable, a dynamic option name such as `curl --$FLAG URL`, an unquoted dynamic option value, or an indirect command prefix, at low effort) follow `options.unanalyzable`:
  - `report` (default): fails as a finding. The message names the job, step, and argument shape, and says `download URL or argument shape` is unanalyzable at effort low. An indirect command says `downloader command is unanalyzable at effort low`.
  - `advise`: a non-failing advisory.
  - `ignore`: omitted.

  Definite violations remain findings under every policy.

### Limits

Shell assignment recovery uses one scope per step or selected script,
including all function bodies. A unique assignment in one function can resolve
a later download in another function, even if the assignment function is never
called.

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
      - env:
          TOOL_URL: https://example.invalid/tool.tgz
        run: |
          curl -fsSL "$TOOL_URL" -o tool.tgz
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

A literal step env binding is also unsafe without verification at max effort:

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - env:
          TOOL_URL: https://example.invalid/tool.tgz
        run: curl -fsSL "$TOOL_URL" -o tool.tgz
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

`wget`, `shasum -a 256 -c`, and a configured helper use the same fix. A stable
`BASE_URL="https://example.invalid/base"` followed by `curl "$BASE_URL/tool.tgz"`
is the same download, even alongside other stable assignments. A value with
an unresolved authority or a command substitution that cannot preserve the
URL's argument role does not establish a static download proof.

A checksum check whose arguments contain command substitution (`$(...)` or backticks), including inside double quotes, cannot establish static integrity verification. Quoted filename references such as `"${ARTIFACT}.sha256"` remain supported
after a proven check option. An unresolved argument before the check option
cannot establish verification because it could terminate option parsing. Use a literal checksum file or another statically analyzable check. Single-quoted substitution text is literal.

## Suppression

`skipAnnotationScope: line` narrows only `integrity-check` annotations. It does
not change the ordinary `no-mistakes-disable-*` directives. Move a legacy
whole-step annotation beside its intended download when opting into line scope.

Use a YAML comment, or a shell comment in a selected source.
`no-mistakes-disable-next-line github-actions-download-integrity`
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
rejects a persisted write token. This rule judges http(s) downloads in
`run` bodies and explicitly selected shell sources.

## Invalid UTF-8

Invalid UTF-8 is decoded lossily and scanned, so a violation next to a stray byte is still reported. A file that cannot be read for any other reason is skipped and produces no finding.
