# `github-actions-job-permissions`

## Why and when

Use this rule to keep a workflow-level write grant from silently applying to
every job. A job that needs a write permission should declare that permission
at the job so other jobs do not inherit it. This also makes the caller's
permission boundary visible when a job calls a reusable workflow.

GitHub lets workflow and job `permissions` set `read`, `write`, or `none` for
token scopes. A write grant includes read access, and `id-token: write` allows
the job to request an OIDC token. When a permissions map is specified, scopes
omitted from that map are set to `none`. See GitHub's [workflow syntax
reference](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#permissions) and [reusable workflow
permissions](https://docs.github.com/en/actions/reference/workflows-and-actions/reusing-workflow-configurations#supported-keywords-for-jobs-that-call-a-reusable-workflow).

## What it catches

The rule examines direct `.github/workflows/*.yml` and
`.github/workflows/*.yaml` files selected by the rule application's shared
`include` and `exclude` filters. It reports each job that inherits an explicit
workflow-level write grant because the job has no `permissions` key. This
includes `write-all`, permission mappings with a `write` value, and
`id-token: write`. Jobs that call reusable workflows with `uses:` are checked
too, because the caller's token permissions are passed to the called workflow
and can only be maintained or reduced there.

A job-level `permissions` declaration prevents this inheritance finding. An
empty map (`permissions: {}`) gives the job no token permissions, and a static
job map such as `contents: read` defines the job's own grant set. The rule does
not compare those explicit job grants against a separate least-privilege
policy.

The rule only reasons from permission declarations in the workflow. If neither
the workflow nor a job declares permissions, it makes no claim about the
effective `GITHUB_TOKEN` permissions: repository, organization, and event
defaults are outside this check.

## Configuration

There are no rule-specific options. The shared `scope`, `projects`, `include`,
and `exclude` fields select applications. By default the rule selects direct
workflow YAML files under `.github/workflows/`.

```yaml
rules:
  - rule: github-actions-job-permissions
    scope: repository
```

## Valid example

The workflow has an explicit write grant, and the job declares its own
permissions instead of inheriting it:

```yaml
permissions:
  contents: write
jobs:
  publish:
    permissions:
      contents: write
    runs-on: ubuntu-latest
    steps:
      - run: echo publish
```

An empty job map also opts that job out of workflow-level grants:

```yaml
permissions:
  id-token: write
jobs:
  test:
    permissions: {}
    runs-on: ubuntu-latest
    steps:
      - run: echo test
```

## Counterexample

`id-token: write` is a write grant. Since `test` has no job-level permissions,
it inherits the grant:

```yaml
permissions:
  id-token: write
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo test
```

The same applies to reusable-workflow callers:

```yaml
permissions:
  packages: write
jobs:
  publish:
    uses: example-org/workflows/.github/workflows/publish.yml@v1
```

## Fix

Declare permissions on each job that needs a write grant. For a job that needs
only to read repository contents, for example:

```yaml
permissions:
  packages: write
jobs:
  test:
    permissions:
      contents: read
    runs-on: ubuntu-latest
```

Use `permissions: {}` for a job that needs no `GITHUB_TOKEN` access. Reusable
workflow callers should declare the minimum grant their called workflow needs;
the called workflow cannot elevate it.

## Analysis and uncertainty

The rule reports only definite writes that are inherited from an explicit
workflow permission declaration. It does not infer a default when the workflow
permission block is absent. Expressions, null or malformed permission values,
unresolved YAML merges, malformed YAML, unreadable files, and job structures
the analyzer cannot interpret are unresolved rather than silently treated as
safe.

Configure the shared `unanalyzable` policy as `report` (the default), `advise`,
or `ignore`. `report` returns unresolved cases as failing findings, `advise`
returns them as non-failing advisories, and `ignore` omits those uncertainty
diagnostics. Definite inherited writes remain findings under every policy.

## Suppression

Use YAML comments with `no-mistakes-disable-line` or
`no-mistakes-disable-next-line` to suppress a finding at its reported line, or
`no-mistakes-disable-file` to suppress findings for the whole workflow file.
Inherited-write findings are associated with the affected job; unresolved
permission findings are associated with the relevant declaration when one is
available.

```yaml
permissions:
  id-token: write
jobs:
  # no-mistakes-disable-next-line github-actions-job-permissions
  test:
    runs-on: ubuntu-latest
```
