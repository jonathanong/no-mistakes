# Planning and graph APIs

The `dependencies`, `dependents`, and `related` options accept
`relationships: ["call"]` to select the opt-in lexical call edges. Calls are
resolved only for local functions, direct named imports, static namespace-member
imports such as `api.run()` from `import * as api`, and explicit named re-exports.
`depth: 1` selects direct calls and `depth: 0` selects none;
computed/dynamic callees, globals, and ambiguous re-exports are omitted rather
than guessed. The default relationship set and `"all"` intentionally exclude
call edges.

`writePlanningImpactArtifacts(options)` is an integration helper for callers
that need private, CLI-compatible planning files without spawning several
commands. It runs one prepared `analyzeProject()` request, then writes
`dependencies`, `dependents`, `symbols`, and Vitest `plan` artifacts as JSON,
stderr, and status files in `outputDirectory`. `changedFilesManifest` is the
path to a private regular file directly in that same existing `0700` directory
(exactly `0700`, with no additional permission or special bits); its contents
are newline-delimited literal repository-relative paths (only empty records are
ignored) and its filename cannot collide with an artifact destination. The
directory must remain the same private directory for the duration of the
operation. It must share its parent's filesystem (mount boundaries are
rejected), and that parent must permit atomically renaming the directory:
publication parks and restores it. Bind mounts or other directories the
platform cannot rename are unsupported and fail during that atomic parking.
Artifact files use mode `0600`;
all four existing statuses are first made nonzero before analysis begins, and
failures remove stale JSON and attempt to write bounded diagnostics and nonzero
statuses for every report. Those failure artifacts are best effort: inability
to publish them never masks the original analysis or publication error. If that
reporting transition cannot restore the output directory, the original error's
non-enumerable `failureReportingError` contains the parked recovery path; a
non-extensible thrown value is instead preserved in an `AggregateError`. The
helper serializes concurrent calls that resolve to the same output directory,
covering invalidation, analysis, and publication so an older call cannot mark
its artifacts successful while a newer analysis is pending. Serialization uses
an OS advisory lock shared by Node processes and worker isolates. Its adjacent
`.<output-name>.planning-impact.lock` file is private and empty; it stores no
analysis state, and process or worker-isolate exit releases the lock. Real paths
and symbolic-link aliases use the same canonical lock identity. Changed-files
manifests must be valid UTF-8; a leading UTF-8 BOM is ignored. Safe manifest
resolution failures also replace stale success artifacts with failure output.
The helper is unavailable on Windows because
Node does not expose a trustworthy private Windows ACL check. Structural paths
are sent to traversal reports as `{ file }` entries, so `#` remains part of a
literal filename rather than a symbol delimiter. Deleted or renamed-away
structural paths remain in traversal and test-planning inputs, but are omitted
from the symbols input because no source file remains to analyze.

```js
await writePlanningImpactArtifacts({
  root: process.cwd(),
  changedFilesManifest: "/private/run/changed-files.txt",
  outputDirectory: "/private/run",
  broad: false,
});
```

The npm package also exposes this integration as `no-mistakes planning-impact
--changed-files <manifest> --output-dir <directory>`. Its current working
directory is `root`; `--broad`, `--timeout`, `--invocation-timeout-ms`, `--lock-timeout`,
`--fail-on-lock`, `--jobs`, and `--profile ci` map to the matching API options.
A later API `jobs` value that cannot resize the process pool surfaces optional
`jobsPoolWarning` instead of a new worker count; see Invocation Lock And Timeouts.
The command is silent on success. Argument and analysis failures write a
UTF-8-safe diagnostic of at most 4 KiB to stderr and exit `1`. It is
intentionally unavailable from the Cargo-installed native binary.

The [runtime inventory](../node-api.md#runtime-exports) is the complete runtime export surface. Keeping this
list exhaustive makes a newly added function visible to agents even when it
does not have a one-to-one CLI command. Analysis JSON objects may include
optional `jobsPoolWarning` (`JobsPoolWarning`), exposed by the exported
`WithInvocationResult<T>` type, when a later `jobs` value cannot
resize the process pool; the field is omitted otherwise. Arrays, scalar results,
source-only SQL parsing/extraction, and diagnostics polling do not add this metadata.

The package also exposes its optional Vitest integration through
`no-mistakes/test-timeout-policy` and
`no-mistakes/test-timeout-policy/plugin`. The `/plugin` entry provides Vite
plugins and imports the optional Vitest peer. See the
[timeout policy guide](../test-timeout-policy.md) for configuration and adoption
details.

| Runtime export | API | Import |
| --- | --- | --- |
| `assertTestTimeout` | `assertTestTimeout(value, label, maximum?)` | `no-mistakes/test-timeout-policy` |
| `assertTimeoutConfig` | `assertTimeoutConfig(config, maximum?)` | `no-mistakes/test-timeout-policy` |
| `snapshotTimeoutOptions` | `snapshotTimeoutOptions<T extends object>(options: T, label, maximum?, nullInherits?): T` | `no-mistakes/test-timeout-policy` |
| `protectTimeoutConfig` | `protectTimeoutConfig(config, maximum?)` | `no-mistakes/test-timeout-policy` |
| `guardRegistration` | `guardRegistration<T extends Registration>(original: T, kind: Kind, maximum?): T` | `no-mistakes/test-timeout-policy` |
| `guardRuntimeConfig` | `guardRuntimeConfig<T extends object>(api: T, maximum?): T` | `no-mistakes/test-timeout-policy` |
| `guardVitestExports` | `guardVitestExports<T extends object>(api: T, maximum?): T` | `no-mistakes/test-timeout-policy` |
| `guardPublicRunner` | `guardPublicRunner<T extends RunnerConstructor>(original: T, maximum: number, protect: Protector<InstanceType<T>>): T` | `no-mistakes/test-timeout-policy` |
| `protectRunnerTimeouts` | `protectRunnerTimeouts(runner: { config: object }, maximum?): void` | `no-mistakes/test-timeout-policy` |
| `testTimeoutPolicyPlugin` | `testTimeoutPolicyPlugin(maximum?, registrationPackages?, nodeRunner?: NodeRunnerPolicy)` | `no-mistakes/test-timeout-policy/plugin` |
| `browserTestTimeoutPolicyPlugin` | `browserTestTimeoutPolicyPlugin(maximum?, registrationPackages?)` | `no-mistakes/test-timeout-policy/plugin` |

The policy entry also exports the types `Registration`, `Kind`,
`RunnerConstructor` and `Protector<T extends { config: object } = { config: object }>`. `guardPublicRunner`
ties the protector argument to the constructed instance with
`Protector<InstanceType<T>>`. `Kind` is the union `"test" | "suite" |
"hook"`. The plugin entry exports `NodeRunnerPolicy`; its `runner` field names
the absolute resolved module for the final custom Node runner, or `null` to use
Vitest's native runner. These named types are part of the entrypoint contract.

`testsTargets()` and test-plan targets set `workspace: true` when a Vitest
workspace/project-array source must be passed with `--workspace`; the emitted
`runnerArgs` already contain the correct flag. This includes configured and
default-discovered `vitest.workspace.*` and `vitest.projects.*` sources,
including JSON project arrays, matching the CLI. A default-discovered root
workspace/project-array source takes precedence over sibling
`vitest.config.*`; explicitly configured paths remain authoritative.

The Playwright APIs load the same selector-wrapper configuration as the CLI.
Configured wrapper calls therefore appear in `playwrightEdges()` and
`analyzeProject()` through the existing selector-edge JSON shape; no separate
Node option or result type is required.

The graph APIs (`dependencies`, `dependents`, `related`, `flow`, and graph
reports in `analyzeProject`) accept the `workflow`, `workflow-job`,
`workflow-step`, `workflow-needs`, `workflow-uses`, `workflow-run`, and
`workflow-artifact` relationship values. `workflow` includes all six edges;
the precise values retain their required structural job/step bridges for a
connected traversal. `all` includes `workflow`.

Workflow jobs and steps are virtual graph nodes with IDs
`workflow.yml#job:<job>` and `workflow.yml#job:<job>/step:<zero-based-index>`.
`DependencyFile` records expose `workflowFile`, `job`, and optional `step`;
`FlowNode` additionally uses `kind: "workflow-job"` or `"workflow-step"`.
The workflow graph tracks only local, static topology: local reusable workflows
and action descriptors, supported literal `run:` targets/package scripts, and
same-run artifact upload -> download edges. It omits remote `uses`,
`workflow_run`, malformed/dangling endpoints, dynamic shell resolution, and
targets outside the tracked graph universe. `ci` remains the separate legacy
`CiInvocation` relationship from workflow file to supported Rust Cargo binary.

The graph APIs also accept `trpc`. That opt-in relationship follows static
tRPC router procedures and client calls through virtual nodes identified as
`router.ts#procedure:user.get`. `DependencyFile` records expose `routerFile`
and `procedure`; `FlowNode` uses `kind: "trpc-procedure"`. `all` does not
include `trpc`. Empty `projects.*.trpc.routers` lists disable extraction.

`testsPlan(options)` returns `changedFiles`, the sorted, deduplicated
changed-file inventory prepared by that same call, relative to the request root.
The field is present even when no tests are selected and retains deleted paths
plus both sides of detected renames and copies.

Ordinary `testsPlan()` configured `direct` groups select changed tests and
tests one reverse import-family or same-directory `TestOf` edge away from a
changed file. That 1-hop set is selected before `dependencies` and therefore
survives the environment file/percent limit. Markdown, resource, route, and
multi-hop import paths stay in `dependencies`. This is distinct from
`directTestOwner`.

Set `directTestOwner: true` with an explicit `framework` to select only changed
framework-owned tests and framework-owned tests one reverse canonical graph edge
away. This bypasses test-plan environment policy (including groups, limits,
samples, fallback, and include/exclude filtering), attaches normal execution
targets, and returns a `direct-test-owner` group with `fallbackTriggered:
false`. `limitPercent`, `limitFiles`, and `globalConfigFallback` conflict with
this option. `entrypoints` also conflicts with it: direct-owner selection is
bounded to changed files and one reverse canonical graph edge, so use
`testsImpact()` for explicit entrypoint traversal. The returned warnings retain
canonical graph diagnostics for dynamic resource calls in changed files, so
incomplete reverse ownership is visible to API consumers.

The TypeScript declaration models this as a discriminated option: direct-owner
plans require `framework`, while ordinary plans omit `directTestOwner` or set it
to `false`.

Vitest literal setup paths use each effective project root, including paths
without `./`. Package and tsconfig aliases take precedence over relative fallback.

[Node/N-API guide](../node-api.md)
