# `github-actions-cache-key-policy`

## Why and when

A cache key that omits the runner platform, or that can be restored from a shorter prefix, can
reuse bytes built for another operating system, architecture, or lockfile. Enable this rule when a
portable cache path must be keyed by the platform and by a digest of a configured lockfile.

The rule is opt-in. A cache whose `path` is not listed in `caches` is silent, including an
expression path that is not exactly one configured path.

## What it catches

Checks `actions/cache`, `actions/cache/restore`, and `actions/cache/save` in workflows placed
directly under `.github/workflows/` and in composite actions under `.github/actions/**/action.yml`
or `action.yaml`. The action reference is matched without regard to case, an `@ref` suffix, or a
trailing slash. Other actions are silent. Nested workflow files and non-composite action
definitions are ignored. A non-composite action is one whose `runs.using` is not `composite`.

A step is in scope only when a normalized `with.path` entry equals a configured `caches.path`.
Normalization trims whitespace, removes one leading `./`, and removes trailing slashes.
`.cache/widgets` does not match `.cache/widgets-extra`. A block or sequence of paths matches when
any entry equals a configured path. A comma is not a path separator.

For each match, the `key` must contain every configured `keyParts` fragment as its own
`${{ }}` expression token. `matrix.os` does not satisfy `runner.os`. `runner.os_name`,
`runner.os.extra`, `matrix.runner.os`, the literal text `runner.os` outside an expression, and a
string literal such as `${{ 'runner.os' }}` or `${{ "runner.os" }}` do not satisfy `runner.os`.
A fragment on a later line of a plain or double-quoted `key` scalar still counts.

The key must also prove a digest of the configured lockfile, in either of these ways:

- `hashFiles` with that exact path as an argument, and only when `allowHashFiles` is true.
  `hashFiles('orders.lock', 'widgets.lock')` satisfies `widgets.lock`. A glob that might include
  the lockfile does not. `hashFiles` outside `${{ }}` does not count.
- An earlier step in the same job, including an earlier step in the same composite action, has an
  `id`. Its `run` script writes the `GITHUB_OUTPUT` name that the key reads as
  `steps.<id>.outputs.<name>`, and that output's value is a hash of the lockfile. `echo`,
  `printf`, an unquoted heredoc, and a brace group or parenthesized command redirected to `GITHUB_OUTPUT`
  all count. A redirect after the closing `}` or `)` applies to every command in that group
  that does not already redirect standard output, including
  `{ echo "digest=$digest"; } >> "$GITHUB_OUTPUT"`,
  `( echo "digest=$digest" ) >> "$GITHUB_OUTPUT"`, and
  `{ echo "digest=$digest"; } >& "$GITHUB_OUTPUT"`. A group with no redirect, or a redirect
  to another file, does not count. The redirect is fixed before the group body runs, so an
  assignment inside the group does not change that destination. Assignments inside a
  parenthesized command do not affect later commands. A command after `;` on the same line
  is a separate command. A `;` inside a nested group stays in that group. The closing `)`
  may share a line with the last command. The closing `}` must be its own command, including
  after `;` or a background `&`. A later pipe discards
  the group unless standard output is redirected to a file before that pipe, and a pipe inside
  `$(...)` or backticks stays in that substitution. A redirect such as `2>` or `3>>` leaves
  standard output alone, so redirecting only those descriptors does not write the digest.
  The last standard-output redirect is the group's destination. A digit run is a descriptor
  only when it is its own word, so `1>> "$GITHUB_OUTPUT"` explicitly redirects standard
  output and `ignored2>/tmp/orders` redirects standard output implicitly. `>=` sends standard
  output to a file named `=`. An unquoted heredoc counts
  only when the command's last standard-output redirect, or the group's, is the output file.
  Assigning another variable from `$GITHUB_OUTPUT`, including `OUT="$GITHUB_OUTPUT"` and a later
  copy of that variable, makes a `>` or `>>` redirect to `$OUT` the same as a redirect to
  `$GITHUB_OUTPUT`. `echo "$OUT"`, `test -f "$OUT"`, and passing `$OUT` as an argument are not
  writes. Single quotes do not expand, so `> '$OUT'` and `> '${OUT}'` are not writes; an unquoted
  or double-quoted `$OUT` or `${OUT}` is. `>&` before a non-numeric target, including
  `>& "$OUT"` and `>&"$OUT"`, is a file redirect. `>&2`, `>& 2`, and `2>&1` are not. A suffix on the
  redirect word, such as `"$OUT".bak` or `"$OUT"bak`, is part of the filename and is not
  another variable. An escaped `\$` is literal, so `>\$GITHUB_OUTPUT` is not a write. Empty
  quotes do not make a single-quoted `$OUT` expand, so `> '$OUT'""` is not a write. Joining
  `"$"` to a name, as in `>"$"GITHUB_OUTPUT`, is not an expansion. A quoted piece of a
  redirect name, as in `>out" widgets.lock"`, is part of that filename. The next
  `>` starts another redirect, so `>/tmp/orders>$OUT` still writes `$OUT`. A `<<<` redirect is
  not a heredoc, and `<<` inside `$(( ))` is not a heredoc, so later
  lines are still read. A variable assigned from `sha256sum widgets.lock` (or another checksum
  command that names the lockfile in that same simple command) and then written to that output
  counts. `sha256sum widgets.lock | awk '{print $1}'` counts because the checksum command names
  the lockfile. `sha256sum orders.lock | grep widgets.lock`, `cat widgets.lock | sha256sum`, and
  `sha256sum orders.lock & cat widgets.lock` do not. `sha256sum widgets.lock & echo done` still
  counts. `sha256sum 2>&1 widgets.lock` counts. An `&` that belongs to a redirection (`2>&1`,
  `>&2`, `&>`, `&>>`, `<&`) does not split the command. A standalone `&` does, and `&` inside
  `$(( ))` does not. A digit prefix and a quoted suffix are one word, so
  `>&1"widgets.lock"` is the file `1widgets.lock`, not fd 1. A command ending in `&`,
  `>&`, `&>`, `2>&`, or `>` has no redirect target, so that ending does not hide the
  lockfile. Reassigning the variable to a
  constant, `cat`, or a hash of a different file drops the proof; an earlier copy into a
  different variable keeps it.
  Reassigning it to a parameter expansion of itself keeps the proof: `"$digest"`, `${digest}`,
  `${digest%% *}`, `${digest%%[[:space:]]*}`, `${digest:0:16}`, and `${digest#*}`. Mixing that
  expansion with other text, or expanding a different variable, drops the proof.
  A later step, another job, or a different output name does not count. The rule does not run
  the script. A full-line `#` comment and a trailing ` #` comment are ignored. `-` is part of a
  path token, so `not-widgets.lock` does not match `widgets.lock`. `/` is a boundary, so
  `dir/widgets.lock` matches.
- A workflow step `uses: ./some/action` can supply the digest when that directory's `action.yml`
  or `action.yaml` is a composite action. The action must declare `outputs.<name>.value` as an
  expression that reads a `run` step output proven in that same action, and the cache key must
  read `steps.<step id>.outputs.<name>` for the workflow step's `id` and that declared output.
  A missing file, a non-composite action, an output with no `value`, a constant, or a nested
  `uses` inside the action is not proof. `..` path segments are ignored.

`rejectRestoreKeys` defaults to true. Any non-empty `restore-keys` value, including a YAML
sequence, is a finding. An empty or missing value is allowed. Set `rejectRestoreKeys` to false to
allow prefix fallbacks.

A missing `key` is reported on the step. Platform and digest gaps are reported on the `key` line.
`restore-keys` is reported on its own line. Invalid YAML is reported at the parser location.
Workflow or action bytes that are not UTF-8 are reported on the line of the first invalid byte.
Anchors, aliases, and other shapes whose cache steps cannot be aligned to ordinary block locations
produce one finding on line 1 when a configured path is involved, and are not partially judged.
A cache step that does not use a configured path stays silent, including when the file uses anchors.

## Options

`caches` is required and must list at least one cache. Each entry has a non-empty `path`, a
non-empty `lockfile`, and at least one dotted `keyParts` fragment such as `runner.os`. The same
normalized path cannot be listed twice. `rejectRestoreKeys` defaults to true. `allowHashFiles`
defaults to false, so a native `hashFiles` call is not provenance until you opt in. There is no
repository-wide cache scan when `caches` is omitted.

```yaml
rules:
  - rule: github-actions-cache-key-policy
    scope: repository
    options:
      rejectRestoreKeys: true
      allowHashFiles: true
      caches:
        - path: .cache/widgets
          lockfile: widgets.lock
          keyParts:
            - runner.os
            - runner.arch
```

## Valid example

With `allowHashFiles` true, the first key carries both platform fragments and `hashFiles`. The
second key uses an earlier step in the same job instead: that step has an `id`, hashes
`widgets.lock`, and writes that digest to `GITHUB_OUTPUT`. Neither step sets `restore-keys`.

```yaml
name: Widgets
on: push
jobs:
  unit:
    runs-on: ubuntu-latest
    steps:
      - id: lock
        run: |
          digest=$(sha256sum widgets.lock | awk '{print $1}')
          echo "digest=${digest}" >> "$GITHUB_OUTPUT"
      - uses: actions/cache@v4
        with:
          path: .cache/widgets
          key: widgets-${{ runner.os }}-${{ runner.arch }}-${{ hashFiles('widgets.lock') }}
      - uses: actions/cache@v4
        with:
          path: .cache/widgets
          key: widgets-${{ runner.os }}-${{ runner.arch }}-${{ steps.lock.outputs.digest }}
```

## Counterexample

This shared key has neither platform fragment nor a lockfile digest, and `restore-keys` can restore
another platform or lockfile from the prefix:

```yaml
name: Widgets
on: push
jobs:
  unit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/cache@v4
        with:
          path: .cache/widgets
          key: widgets-shared
          restore-keys: widgets-
```

## Fix

Put each configured fragment in the key as its own expression, for example `${{ runner.os }}` and
`${{ runner.arch }}`. Add a digest with `hashFiles('widgets.lock')` when `allowHashFiles` is true,
or with a preceding same-job step that writes a hash of the lockfile to `GITHUB_OUTPUT`. Remove
`restore-keys`, or set `rejectRestoreKeys` to false when a prefix fallback is intentional. Rewrite
anchors and aliases that affect a configured path as ordinary block-style steps. There is no
automatic fix: the platform list and the lockfile path come from configuration.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning
(requested effort low; effective effort max because this rule has no low implementation) and runs
the rule at `max`.

## Suppression

Each finding line is suppressed on its own. A directive on `key` does not suppress `restore-keys`.

```yaml
# no-mistakes-disable-next-line github-actions-cache-key-policy
key: widgets-shared
# no-mistakes-disable-next-line github-actions-cache-key-policy
restore-keys: widgets-
```

Use `# no-mistakes-disable-line github-actions-cache-key-policy` on the finding line, or
`# no-mistakes-disable-file github-actions-cache-key-policy` at the top of the file. For a flow
step, the findings share the step line, so one directive on that line covers them.

## Related rules

- [`github-actions-pinned-hash`](github-actions-pinned-hash.md) owns action reference SHAs.
- [`github-actions-script-expressions`](github-actions-script-expressions.md) owns untrusted expressions inside scripts.
- [`github-actions-download-integrity`](github-actions-download-integrity.md) owns `curl` and `wget` checksums.
