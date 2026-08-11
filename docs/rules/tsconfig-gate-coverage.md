# `tsconfig-gate-coverage`

Requires every tracked `tsconfig.json` or `tsconfig.*.json` outside
`node_modules` to be registered in both a configured GitHub Actions workflow
and a local whole-project typecheck command.

```yaml
checks:
  commands:
    - name: typecheck-web
      command: [pnpm, --dir, web, exec, tsc, --noEmit]
      fileArgs: none
      always: true

rules:
  - rule: tsconfig-gate-coverage
    scope: repository
    options:
      allowProjects:
        web/tsconfig.dependency-cruiser.json: Used only by dependency-cruiser, not tsc.
```

The rule recognizes static `tsc --noEmit` commands in workflow `run:` steps
and `checks.commands`. It supports `--project <path>` and `-p <path>`, a
default `tsconfig.json` relative to the effective
working directory, sequential `cd` commands, and
`pnpm --dir <path> exec tsc`. Workflow working directories
may come from workflow/job `defaults.run.working-directory` or a step's
`working-directory`.
Step-based jobs need a non-empty, statically resolvable `runs-on` string,
label array, or `group`/`labels` mapping. Static matrix and reusable-input
runner selectors are resolved per generated job before runner platform and
implicit-shell checks; unresolved selectors do not provide coverage.
Repository-local action steps (`uses: ./path`) count only when the tracked
target directory contains parseable `action.yml` or `action.yaml` metadata
with the required name, description, and a supported `runs` contract. A
JavaScript action's `runs.main` must resolve to a tracked file under that action
directory. Local targets are checked in step execution order, so a statically
skipped job or step does not invalidate an independent typecheck, while a
missing action prevents later commands in the same executed job from counting.
Static local reusable-workflow jobs (`uses: ./.github/workflows/*.yml`) are
followed transitively and their step-based jobs are evaluated under the direct
caller's file triggers. Remote, dynamic, missing, non-callable, and cyclic
calls do not provide coverage. One complete, enforcing, acyclic caller path is
sufficient; partial coverage from separate caller paths is never combined.
Every declared reusable call, including a statically skipped call, still
participates in cycle, nesting-depth, and unique-target validation; skipped
callees are validated without crediting their commands.
For each direct workflow and triggering event, reusable activation evaluates at
most 1,024 distinct path-sensitive input states. A graph that exceeds this
budget provides no coverage for that root event, rather than allowing layered
branching to consume unbounded analysis resources.
The containing workflow must declare at least one file-triggered `push`,
`pull_request`, or `pull_request_target` event whose path filters allow every
visible TypeScript/JavaScript source selected by that project's
`files`/`include`/`exclude` settings. Projects with no known source files fall
back to the tracked tsconfig path. Input values and path coverage are evaluated
for each direct caller event independently, so coverage from different events
is never combined. An explicitly activity-filtered `pull_request` or
`pull_request_target` event must include `synchronize`, the event that runs when
source commits are added to an open pull request. Manual, scheduled, reusable,
empty, tag-only, and path-filtered-out workflows cannot provide a repository
typecheck gate on their own. A `workflow_call` workflow can provide one when
reached from an applicable caller. For
example, `paths: [app/tsconfig.json]` cannot cover `app/src/index.ts`; add
`app/**` or an
unfiltered applicable event. Inclusive `branches`, `tags`, or `paths` filters
that use `!` exclusions must also contain at least one positive pattern, as
required by GitHub Actions.

Workflow commands run only when their effective shell is GitHub Actions'
implicit shell or a static `bash`/`sh` form. The rule honors workflow and job
`defaults.run.shell` plus a step-level `shell` override; static shell templates
must invoke `bash` or `sh`, pass the script as `{0}`, and use only
execution-preserving flags: `-e`, `-u`, `-x`, and Bash's `-o pipefail`,
`--noprofile`, and `--norc`. This includes GitHub Actions' standard
`bash --noprofile --norc -eo pipefail {0}` and `sh -e {0}` templates.
Context-free literal shell expressions are reduced before this classification;
shell expressions whose value remains dynamic do not count.
Other shells (such as `python`, PowerShell, or `cmd`) and dynamic/custom shell
forms do not count; neither do non-executing modes such as `bash -n {0}`.
Implicit and built-in `bash`/`sh` shells propagate failures. Custom templates
must include `-e` or `-o errexit` to credit a typecheck before a later command;
without it, only a final `tsc` command counts.
An implicit shell does not count for statically Windows-labeled runners
(`windows-*` or a self-hosted `windows` label), because GitHub Actions defaults
those runners to PowerShell; specify a supported `bash` or `sh` shell instead.
A bare `self-hosted` label is also rejected with an implicit shell because its
operating system is not statically known; add a `linux`/`macos` label or an
explicit supported shell.
Jobs that declare `container` or `services` count only with a statically Linux
runner label. GitHub does not support those fields on Windows or macOS runners,
and an unknown custom runner label cannot prove the required Linux host.

Literal YAML `if: false` and `continue-on-error: true` values, plus exact
constant expressions `${{ false }}` and `${{ true }}`, on a workflow job or
step do not count as CI registrations because they cannot enforce a typecheck.
For reusable calls, the rule also resolves exact boolean input references and
negations/comparisons in call-job, callee-job, and step conditions. It also
short-circuits logical `&&` and `||` expressions when a known input operand
determines the result. Literal call inputs, declared defaults, omitted values,
and exact `${{ inputs.name }}` forwarding preserve boolean, string, and number
values through transitive calls. This lets the rule resolve exact string
equality/inequality and number equality/inequality or relational comparisons,
as well as input truthiness. Static `contains`, `startsWith`, and `endsWith`
calls are also resolved using GitHub's string coercion; missing properties
coerce to an empty string. Static `case` calls select the first truthy branch
or their default, while an unknown predicate remains unresolved. Expressions
whose result remains dynamic fail open as potentially runnable.
Reusable input default expressions must match their declared scalar type and
may use only `github`, `inputs`, and `vars`; malformed defaults or unavailable
contexts invalidate the workflow before any command can provide coverage.
Defaults can read caller event state and input values resolved earlier in the
contract's canonical input order. References whose value is not yet resolved
remain dynamic rather than being guessed.
For static matrices, `${{ matrix.name }}` bindings, step conditions, and job or
step `continue-on-error` expressions are evaluated once per generated
combination after `exclude` and ordered `include` expansion. Execution and
failure tolerance therefore stay correlated: a typecheck that runs only in an
allowed-to-fail combination does not count. Literal complete expressions in
static `include` and `exclude` entries retain their YAML scalar types; an entry
whose value remains dynamic stops static enumeration conservatively. A missing
property in a statically known matrix coerces to an empty string. Dynamic
matrices and their properties remain unresolved and fail open as potentially
enforcing, but a root matrix expression whose parser result is guaranteed
scalar is rejected because Actions requires an object. Dot and single-quoted
bracket property access share the same normalized parser for conditions and
reusable-input forwarding.
Condition expressions must also use contexts available at their location.
For example, job conditions cannot read `secrets`, while step conditions can
read `steps`, `runner`, and `env`. A malformed or unavailable context prevents
the workflow from providing coverage. Step conditions merge static environment
values with GitHub's workflow, job, then step precedence and string coercion;
an omitted reusable secret referenced by an environment value resolves to the
empty string. Workflow `defaults.run` values must be static; job defaults may
use their documented `github`, `needs`, `strategy`, `matrix`, `env`, `vars`,
and `inputs` contexts. Workflow concurrency expressions may use `github`,
`inputs`, and `vars`; job concurrency additionally permits `needs`, `strategy`,
and `matrix`. Job and step `continue-on-error` expressions, plus environment names
and URLs, use their own GitHub context/function sets; status functions are not
accepted in `continue-on-error`. Strategy `fail-fast` expressions use the
documented strategy contexts and must be boolean when their result type is
statically known; `max-parallel` must similarly be a positive integer.
Reusable-input `max-parallel` expressions are rechecked with the active input
values, so a value that resolves to zero or a non-integer cannot provide
coverage.
Job and step `timeout-minutes` expressions use their documented context sets,
must resolve to positive integers, and do not admit status functions. Only
step-level timeouts admit `hashFiles` and enforce the documented 360-minute
maximum; job timeouts may be larger and are ultimately bounded by the selected
runner.
Workflow-dispatch choice options/defaults and container/service port
declarations must also match their field-specific GitHub types and static
Docker shape. Complete expressions in supported port components remain dynamic
rather than being rejected as malformed mappings. Container and service image,
environment, port, volume, option, and registry-credential expressions each
accept only the contexts GitHub exposes at that field; a step-only context
cannot validate an earlier container configuration. Runner labels and
container/service images likewise reject contexts and functions unavailable at
their own fields. A fully static image must be a valid Docker reference; an
image that remains dynamic, including one that depends on a dynamic matrix,
does not earn typecheck coverage because its Docker reference cannot be
validated. Resolved registry usernames and passwords must be non-empty;
available secret values remain opaque, while an omitted reusable secret
resolves empty and cannot start the container. Container options reject
GitHub's unsupported `--network` and `--entrypoint` flags, and service options
reject `--network`; dynamically unresolved options remain conservative.
Container and service volumes are likewise revalidated after static matrix and
reusable-input substitution, so a resolved bind source must be absolute and a
resolved named volume must satisfy Docker's volume-name shape.

Reusable-workflow secret validation follows each call edge. A directly
triggered workflow can inherit its available repository or organization
secrets; `secrets: inherit` preserves that availability through nested calls,
while an explicit secret mapping narrows it to the destination names supplied
by that mapping. Required secrets must therefore be available at the immediate
caller boundary rather than being inferred from an earlier ancestor.

The rule evaluates the successful gate path: `success()`, `always()`, and
`!cancelled()` are runnable there, while `failure()` and `cancelled()` are not.
This prevents failure-handler or cancellation-only type checks from satisfying
the required CI gate.

A job blocked by a statically skipped `needs` dependency, including a
transitive dependency, does not count. A job condition that contains a status
check such as `always()` or `!cancelled()` can explicitly continue after a
skipped need when the whole condition is statically true. A dependency with
`continue-on-error: true` is non-enforcing itself but is not treated as skipped
for downstream jobs.

A project whose effective local `compilerOptions.noCheck` is `true` does not
count as typechecked, even when both commands are registered. Remove or disable
`noCheck`, or document an intentional non-typechecking project with
`allowProjects`. Local and installed-package `extends` chains are resolved
through the prepared source store; unresolved configs are left for `tsc` to
reject.

Counterexample: `packages/api/tsconfig.json` exists, but its `tsc --noEmit`
command appears only in a local command catalog. CI can therefore miss type
errors in that package.

Fix: add the matching static typecheck command to a configured workflow and a
`checks.commands` entry with `always: true` and `fileArgs: none`.
Auxiliary configs that intentionally are not compiler projects need a
non-empty reason in `options.allowProjects`; stale, blank, invalid, or
normalization-colliding entries fail the rule.

Dynamic shell expansion, command substitution, arbitrary wrapper scripts,
paths outside the repository, and other unresolved command forms do not count
as registrations. Express such a command statically if it is intended to
provide this gate.
Shell bodies containing `exit`, `return`, `false`, or a failure-mode mutation
such as `set +e` are also rejected as a whole because the rule does not model
shell reachability or option state. Negated pipelines and bodies with unquoted shell comments,
quoted command separators, or shell function/group braces, and local shell
invocations that enable a
non-executing mode such as `bash -n`, are rejected rather
than credited heuristically. A typecheck before another command in an `&&`
list is rejected when a later top-level command could mask a failed or skipped
typecheck. A final static `&&` list remains recognized.

Informational, setup, or config-bypassing commands (`--showConfig`,
`--help`/`-h`, `--version`/`-v`, `--init`, enabled `--noCheck`,
`--listFilesOnly`, and `--ignoreConfig`) do not count, even when combined with
`--noEmit`, because they do not fully typecheck the project. Explicit
`--noCheck false` and `--noCheck=false` forms remain typechecking modes.

Findings use line 1 of the tsconfig, workflow, or configuration file. Use a
top-of-file `no-mistakes-disable-file tsconfig-gate-coverage` directive only
when an intentional exception cannot be represented with a reasoned
`allowProjects` entry.
