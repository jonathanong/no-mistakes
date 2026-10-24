<!-- cspell:ignore nerdctl podman setsid tlsverify -->

# `github-actions-docker-host-ports`

## Why and when

Use this rule when a workflow or composite action starts a container with
`docker run` on a hosted or shared runner. A publish mapping that names a
literal host port (`5432:5432`, `127.0.0.1:5432:5432`) binds that port on the
runner. The next job that asks for the same port fails. Docker can allocate
the host port instead.

The rule is opt-in. Listing it is what turns it on. It is off by default and
is not part of a recommended or strict preset. It reports only a mapping it
can judge. A dynamic publish word stays silent. These forms are not judged:
an `ssh` remote command, an unknown flag before `-p` or `--publish`, and a
YAML merge key (`<<`), which is not applied. Engine-specific global flags
before `run` (`podman --remote`, `nerdctl --namespace`) are not judged. A
literal command inside `bash -c`, `sh -c`, `zsh -c`, or `eval` is judged,
including when `sudo` precedes that shell.

## What it catches

The rule reads workflow files directly under `.github/workflows/*.{yml,yaml}`
and composite `action.yml` / `action.yaml` files under `.github/actions/`. A
file in a subdirectory of `.github/workflows/` is not a workflow GitHub runs,
so it is ignored. Inside those files it reads `run:` strings on workflow job
`steps` and on composite `runs.steps`. Other action kinds are skipped, because
GitHub does not execute `runs.steps` unless `using` is `composite`.

It flags `-p` and `--publish` when the host port is a literal number or a
literal range (`5432`, `8000-8002`), including an IP prefix
(`127.0.0.1:5432:5432`, `[::1]:5432:5432`) and a `/tcp` or `/udp` suffix. These forms are allowed:

- Docker-allocated ports: `127.0.0.1::5432`, `::5432`, `[::1]::5432`,
  `0:5432`, `127.0.0.1:0:5432`, and a container port alone (`5432`).
- Variable host ports: `$PORT:5432`, `${PORT}:5432`, and `${{ ... }}:5432`.

The command word is `docker`, `podman`, or `nerdctl` (a path whose last
segment is one of those counts) followed by `run` or `create`, including
`container run` and `container create`. A leading `sudo` (including
`--`, `--user`, `--preserve-env`, and boolean flags such as `-i`), `env`
(including a path, `-C`, and `-S`), `timeout` (including `-s` and `-k`),
`xargs`, an assignment, or a shell prefix (`if`, `time`, `exec`, `setsid`,
`(`, and the same family) does not hide the command. `bash`, `sh`, and `zsh`
with `-c`, `-lc`, or `-O`, and `eval`, are read again as a script when the
operand is a literal word. Client globals
before the subcommand (`--context`, `-c`, `--tls`, `--tlsverify`, `--host`,
`-H`, `--config`, `--log-level`, `-l`, and the `--tls*` file flags) are
skipped. Flags before the image are parsed when this rule knows them. An
unknown `--name=value` flag is skipped. Any other unknown flag stops the
scan, and publish flags already seen stay findings. A command
substitution or backtick is judged when the publish word itself is literal;
a substitution used as the publish value is not a finding. A heredoc body is
not scanned, including when a space precedes the delimiter. A `docker run`
after the terminator, or on the operator line after `|` or `&&`, is judged.
A `run` value that
is not a string is not a finding. A `-p` after the image is the container
command, not a publish mapping. Invalid YAML is not a finding. Flow-style
steps that the line scan cannot place are reported on line 1.

## Options

This rule has no options. There is no configurable default. Listing the rule
is what turns it on. Product defaults do not enable it.

```yaml
rules:
  - rule: github-actions-docker-host-ports
    scope: repository
```

## Valid example

Let Docker choose the host port.

```yaml
jobs:
  db:
    runs-on: ubuntu-latest
    steps:
      - run: docker run -p 127.0.0.1::5432 postgres
      - run: docker run -p 0:5432 postgres
```

A container port alone also asks Docker to allocate the host port:

```yaml
      - run: docker run --publish 5432 postgres
```

## Counterexample

This mapping pins host port 5432 on the runner:

```yaml
jobs:
  db:
    runs-on: ubuntu-latest
    steps:
      - run: docker run -p 5432:5432 postgres
```

## Fix

Publish with a Docker-allocated host port:

```yaml
      - run: docker run -p 127.0.0.1::5432 postgres
```

`::5432`, `0:5432`, and a container port alone are the same fix. A variable
host port (`$PORT:5432`, `${PORT}:5432`, `${{ env.PORT }}:5432`) is not judged:
the rule cannot tell whether the variable holds a fixed port. The same rewrite
applies to `--publish`, `docker create`, and composite `runs.steps`. Literal
`ports:` on a job `services:` entry or `container:` are judged the same way,
as one scalar, a flow list, a block list, or a block list written in the
same column as `ports:`. A flow mapping (`services: { pg: { ports: [...] } }`)
is judged too. `options:` on a service is read for `-p` and `--publish`.
An empty list item is not a publish. A container port alone, including a
flow list such as `ports: [6004]`, is not a finding.

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Following [recall over precision](../../README.md#how-analysis-works), nothing this rule cannot prove is treated as clean, and it has no such inputs.

- This rule has no unanalyzable inputs: it judges literal Docker port mappings in parsed workflow input. Nothing is routed through `options.unanalyzable`.

## Suppression

Use a YAML comment. `no-mistakes-disable-next-line github-actions-docker-host-ports`
suppresses the next line. `no-mistakes-disable-line github-actions-docker-host-ports`
suppresses the same line. `no-mistakes-disable-file github-actions-docker-host-ports`
in the leading comment block suppresses the file.

```yaml
# no-mistakes-disable-next-line github-actions-docker-host-ports
- run: docker run -p 5432:5432 postgres
- run: docker run -p 5432:5432 postgres # no-mistakes-disable-line github-actions-docker-host-ports
```

When a step's `-` item sits in the same column as `steps:`, the finding is on
that step line, so `no-mistakes-disable-next-line` above the step applies.

## Related rules

[`github-actions-script-expressions`](github-actions-script-expressions.md)
rejects untrusted expressions inside `run` scripts.
[`github-actions-checkout-credentials`](github-actions-checkout-credentials.md)
flags a checkout that persists a write token. This rule judges literal host
ports on `docker`, `podman`, and `nerdctl` `run` and `create` publish
mappings, and on job `services:` and `container:` `ports:`.
