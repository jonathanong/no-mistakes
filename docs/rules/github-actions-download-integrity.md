# `github-actions-download-integrity`

## Why and when

Use this rule when a workflow or composite action downloads a file with
`curl` or `wget` on a hosted runner. An `http` or `https` URL that the same
step does not verify can change between runs. The rule is opt-in. Listing it
is what turns it on. It is off by default and is not part of a recommended or
strict preset.

## What it catches

The rule reads workflow files directly under `.github/workflows/*.{yml,yaml}`
and composite `action.yml` / `action.yaml` files under `.github/actions/`. A
file in a subdirectory of `.github/workflows/` is not a workflow GitHub runs,
so it is ignored. Inside those files it reads `run:` strings on workflow job
`steps` and on composite `runs.steps`.

It flags `curl`, `wget`, and any command named in `downloadHelpers` when a
word is an `http://` or `https://` URL. These downloads are allowed:

- The same step runs `sha256sum -c`, `sha256sum --check`, or
  `shasum -a 256 -c`.
- The host, with its port removed, is in `trustedHosts`.
- The URL path is a checksum file (`.sha256`, `.sha256sum`, or `SHA256SUMS`).
- The step carries `# integrity-check: skip reason=<slug>`. The slug starts
  with a letter or digit and then uses letters, digits, `_`, or `-`.

A simple uppercase assignment such as `BASE_URL="https://example.com"` is
expanded in later double-quoted and unquoted text in that step, so
`curl "$BASE_URL/a.tgz"` is judged as that URL. Single quotes are left
unchanged. A command substitution, backtick, or URL the rule cannot see stays
silent. Invalid YAML is not a finding. A non-composite action is skipped.

## Options

`trustedHosts` defaults to `localhost`, `127.0.0.1`, `api.github.com`, and
`objects.githubusercontent.com`. Setting it, including to `[]`, replaces that
list. `downloadHelpers` defaults to `[]` and adds command names next to `curl`
and `wget`.

```yaml
rules:
  - rule: github-actions-download-integrity
    scope: repository
    options:
      trustedHosts:
        - localhost
        - 127.0.0.1
        - api.github.com
        - objects.githubusercontent.com
      downloadHelpers:
        - ci_download_to
```

## Valid example

Verify the download in the same step, or use a trusted host.

```yaml
jobs:
  tools:
    runs-on: ubuntu-latest
    steps:
      - run: |
          curl -fsSL https://example.com/a.tgz -o a.tgz
          sha256sum -c a.sha256
      - run: curl -fsSL https://api.github.com/repos/example/tool
```

## Counterexample and fix

This step saves a tarball and never checks it:

```yaml
jobs:
  tools:
    runs-on: ubuntu-latest
    steps:
      - run: curl -fsSL https://example.com/a.tgz -o a.tgz
```

Fix: run `sha256sum -c` or `shasum -a 256 -c` in that step. A host you trust
can be added to `trustedHosts`. A download that cannot be checked can carry
`# integrity-check: skip reason=vendor-tarball` on the step.

## Suppression

Use a YAML comment. `no-mistakes-disable-next-line github-actions-download-integrity`
suppresses the next line. `no-mistakes-disable-line github-actions-download-integrity`
suppresses the same line. `no-mistakes-disable-file github-actions-download-integrity`
suppresses the file. On a compact step list, put the next-line comment on the
line above the `- run:` item. `guardrails-disable-*` is not a suppression.

```yaml
    steps:
    # no-mistakes-disable-next-line github-actions-download-integrity
    - run: curl -fsSL https://example.com/a.tgz -o a.tgz
```
