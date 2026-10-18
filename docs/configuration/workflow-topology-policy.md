# Workflow secret inventory

[`workflow-topology-policy`](../rules/workflow-topology-policy.md) can compare
static GitHub Actions secret references with an explicitly declared inventory.
It checks names only. It never contacts GitHub or reads secret values from the
environment.

## Configure the inventory

Add `secretInventory` to a rule application's `options`:

```yaml
rules:
  - rule: workflow-topology-policy
    scope: repository
    options:
      secretInventory:
        repository: [DEPLOY_TOKEN]
        environments:
          production: [RELEASE_KEY]
        # Defaults to [GITHUB_TOKEN]; [] disables all built-in exemptions.
        builtInExemptions: [GITHUB_TOKEN]
        reportUnused: true
```

`repository` lists secret names that are not environment secrets. That includes
repository secrets, organization secrets, and Dependabot secrets. Do not put
those names under `environments`. `environments` maps literal environment names
to names available in that environment. Secret and environment-name matching is
ASCII case-insensitive. Duplicate names within a scope and duplicate
environment keys after case folding are configuration errors. Names must start
with an ASCII letter or underscore and contain only ASCII letters, digits, or
underscores. A name may be declared at repository and environment scope: for a
job in that environment, the environment declaration shadows the same
repository name.

`secretInventory` omitted or set to `null` disables inventory checking. An
empty object enables a strict empty inventory while retaining the default
`GITHUB_TOKEN` exemption. Use `repository: []`, `environments: {}`, and
`builtInExemptions: []` to reject every referenced name.
`reportUnused` defaults to `false`; turn it on to report declared names that
the complete analysis proves unused.

## Name and environment resolution

A workflow, job, or step reference such as `${{ secrets.DEPLOY_TOKEN }}` is
checked against its scope. Jobs without an environment use repository names.
For a job with a literal environment, that environment's declaration takes
precedence for a same-named secret and a missing environment name falls back to
the repository inventory.

Workflow-level `env` is evaluated once when the run is queued, before any job
starts. It is checked only against `repository`. A job `environment:` does not
apply to that map, even when every job uses the same environment. The
[secrets reference](https://docs.github.com/en/actions/reference/secrets-reference)
states: "Organization and repository secrets are read when a workflow run is
queued, and environment secrets are read when a job referencing the environment
starts." The workflow `env` key is separate from `jobs.<job_id>.env` in the
[context availability](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#context-availability)
table and is not re-evaluated per job. Environment secrets are only available
to workflow jobs that reference the environment.

Reusable workflow contracts define aliases, not inventory entries. If a caller
maps `DEPLOY_KEY: ${{ secrets.CI_DEPLOY_TOKEN }}`, the caller's source name is
`CI_DEPLOY_TOKEN`; the callee's contract name `DEPLOY_KEY` is not counted as a
repository secret use.

```yaml
jobs:
  release:
    environment: production
    steps:
      - run: deploy
        env:
          KEY: "${{ secrets.RELEASE_KEY }}"
```

Declare `RELEASE_KEY` under `environments.production`, or remove the
reference. An environment expression cannot select a declared environment
statically.

## Incomplete analysis

Incomplete analysis only affects the unused proof. Incomplete findings are
emitted only when `reportUnused` is `true`. With the default
`reportUnused: false`, those inputs do not fail the check by themselves. An
incomplete result never hides a static undeclared `secrets.NAME` reference.

These inputs make unused-name analysis incomplete:

- a dynamic secret selector, or the whole `secrets` context. `secrets['NAME']`
  is static. `secrets["NAME"]` is not: GitHub expressions require single
  quotes, and wrapping a string in double quotes throws
- an expression-valued job `environment:`
- `secrets: inherit`, a reusable call that cannot be resolved locally, or a
  reusable call cycle
- a workflow or referenced composite action that cannot be read or parsed
- a non-literal `environment:` whose name is declared only on some environment
  and not in `repository` (that name is not proven used or unused)

A call to `owner/repo/...@ref` with a static `secrets:` mapping is fully known
for undeclared checks. The mapping expressions are checked, and the call itself
is not incomplete. An explicit empty mapping passes no secrets.

A non-literal `environment:` still sees repository names. A name absent from
`repository` and from every configured environment inventory is undeclared.

Known static names are still checked. `reportUnused` findings are withheld
when that proof is unavailable. Declarations also need a unique source location
in the effective config file. Inherited or ambiguous declaration locations
produce incomplete analysis rather than an unused finding at a guessed
location.

Composite actions cannot read the `secrets` context directly, even when that
name appears in the inventory. Pass a value from the calling workflow through
an action input or environment variable instead. Inventory declarations do
not make direct composite-action access valid. Only composite actions reached
by `uses: ./...` from a workflow, or from an action already reached that way,
are scanned. An unreferenced vendored or fixture `action.yml` is ignored.

Bare `if` expressions are scanned only for real job-level and step-level `if:`
keys. An action input named `if` under `with:` is ordinary text.

Findings use the source file and line of the reference when available. Suppress
an intentional one-line exception with the standard
`no-mistakes-disable-line workflow-topology-policy` directive; see the
[rule suppression guidance](../rules/workflow-topology-policy.md#suppression).

## CLI and Node API parity

The CLI `no-mistakes check`, async Node `check()`, and an async
`analyzeProject()` `type: "check"` report read the same rule configuration and
produce the same inventory findings. There is no separate API option; pass the
config file path as usual:

```js
const { analyzeProject, check } = require("no-mistakes");
const config = ".no-mistakes.yml";
const root = process.cwd();

(async () => {
  const direct = await check({ root, config });
  const batched = await analyzeProject({
    root,
    config,
    reports: [{ type: "check" }],
  });
})();
```

Their check reports contain the standard `RuleFinding` records in `rules`.
The N-API entrypoints and report types do not change.
