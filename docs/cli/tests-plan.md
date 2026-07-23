# `no-mistakes tests plan`

Select tests to run from changed files, diffs, configured environments, and
dependency graph analysis.

```sh
no-mistakes tests plan vitest --base origin/main --format json
no-mistakes tests plan vitest --from-git-diff origin/main...HEAD --format json
no-mistakes tests plan playwright --changed-file web/app/users/page.tsx --format paths
no-mistakes tests plan dotnet --changed-file dotnet-clients/src/App/FeedService.cs --format paths
no-mistakes tests plan vitest --changed-file web/app/users/users.test.ts --format commands
no-mistakes tests plan swift --changed-file backend/api/feeds.mts --format paths
```

Use this for agent test selection before running expensive suites. Inputs can
come from `--base/--head`, `--from-git-diff`, `--changed-file`,
`--changed-files`, `--diff`, `--diff-stdin`, `--diff-command`, or repeatable
`--entrypoint`.

`--from-git-diff <base...head>` is single-argument sugar over `--base`/`--head`
(conflicts with both): it parses a three-dot refspec and runs the identical
`git diff <base>...<head>` lookup. A bare base or a trailing `<base>...` both
default head to `HEAD`. Two-dot refspecs (`<base>..<head>`) are rejected —
`git diff` gives `..` and `...` different comparison bases, so accepting `..`
here would silently desugar to a different diff than the equivalent
`--base`/`--head` flags.

`--base`/`--head` and `--from-git-diff` stream the full unified diff (not just
file names) into the same parser `--diff-stdin` uses, so revision-backed plans
carry identical hunks, rename/delete facts, and selector/route/queue/HTTP
coverage hints (Playwright plans only) as an inline diff — memory is bounded
regardless of patch size. If Git cannot resolve the request, the command
exits nonzero with a stable diagnostic code in stderr rather than silently
returning an empty plan:

- `git-not-a-repository` — `--root` is not inside a Git repository.
- `git-merge-base-unavailable` — `--base`/`--head` (or the equivalent
  refspec) does not resolve to a commit.
- `git-shallow-history` — both refs resolve, but the merge base was cut by a
  shallow fetch (common in CI checkouts); fetch more history
  (`git fetch --unshallow` or a deeper `--depth`).
- `git-exit-failure` — Git failed for another reason; see the embedded stderr.
- `git-malformed-output` — a single diff line exceeded the internal
  pathological-line bound.

Node's `testsPlan()` rejects with the same stable code and message instead of
resolving to an empty plan.

Key options: `--root`, `--config`, `--tsconfig`, `--environment`,
`--limit-percent`, `--limit-files`, `--global-config-fallback`, `--format`, and
`--json`.

Configured `fullSuiteTriggers.projects` entries may use `{ paths, targets }` to
select only the named Vitest or Playwright runner projects. These selections
report `configured-trigger`, keep `fallback_triggered` false, and are filtered
by the selected environment before limits are applied. Legacy boolean and path
list entries still request the framework-wide fallback. Trigger paths support
ordered `!` exclusions and later re-inclusions.

For revision and inline-diff inputs, `.no-mistakes.yml`/`.yaml` changes are
compared semantically per framework. Formatting-only changes do not invalidate
tests, while a change to Vitest configuration does not invalidate Playwright
and vice versa. Inputs that provide only a changed filename, or whose historical
configuration cannot be reconstructed and parsed, conservatively retain the
configured global fallback.

For TypeScript and JavaScript workspaces, omitting `--tsconfig` resolves each
import with the config that owns its importing file. A shared project change can
therefore select tests from every workspace that actually imports it, even when
their path aliases conflict. Supplying `--tsconfig <FILE>` forces one config for
the whole plan and is intended as a debugging or compatibility override.

Dotnet and Swift plans require explicit config to build the native source graph
that maps changed source files to test projects or targets. `tests.dotnet.projects`
or `tests.dotnet.solutions`, and `tests.swift.packages`, are the source-graph
inputs. If native tests are discoverable but the native source/project change
cannot be traced, the plan falls back to the framework-scoped discovered tests
and sets `fallback_triggered` with a `fallback_reason`.

Example native workspace config:

```yaml
tests:
  dotnet:
    solutions:
      - dotnet-clients/App.sln
    projects:
      app:
        project: dotnet-clients/src/App/App.csproj
      app-tests:
        project: dotnet-clients/tests/App.Tests/App.Tests.csproj
        test: true
  swift:
    packages:
      - swift-clients/core
      - swift-clients/ui
```

Keep these paths scoped to the native workspaces you want analyzed; no
repository-wide `.csproj`, `.sln`, or `Package.swift` scan runs by default.

`--format commands` prints the exact runner commands for selected execution
targets. Use it when an agent needs runnable commands instead of test paths or a
structured plan.

Node API: `testsPlan(options)`.

Plans also trace supported literal filesystem resources through `resource`
edges. JSON reasons expose call-site provenance in optional `via_details`,
aligned with `via`; non-JSON formats remain test-only. Dynamic paths, glob
patterns, and cwd values are reported as warnings rather than treated as a
global fallback.
Vitest `setupFiles` and `globalSetup` are included in the test dependency
graph automatically. A change to either configured module, or to a static
import/re-export reachable from one, selects only the tests owned by that
Vitest project. Inline projects inherit root setup fields only with
`extends: true`; otherwise (including the default and `extends: false`) their
own value applies, and `[]` clears it. A config referenced as a string in
`test.projects` is parsed as an independent config and does not inherit the
referencing config's setup fields.
For supported inline objects, a nested `test` object owns `setupFiles` and
`globalSetup`; same-named outer fields are ignored regardless of direct or
static-spread declaration order.

Vitest workspace configs may export a project array directly or through
`defineWorkspace([...])`. With no `tests.vitest.configs`, executable root
`vitest.workspace.*` and `vitest.projects.*` files are discovered by default.
`vitest.workspace.json` and `vitest.projects.json` remain explicit-only and
accept static arrays of inline objects and string project paths/globs. Folder
globs select configs directly inside each matched project root; nested roots
require their own folder glob. Project config-file globs recognize suffixes
such as `vitest.config.unit.ts` and `vite.config.e2e.js`.

Setup values are extracted statically from string and array forms. Dynamic
expressions and unresolved literal modules produce a JSON warning with the
declaring config, field, and project. When such a declaration is relevant, the
plan conservatively selects the affected owner scope (or the discovered Vitest
framework set when ownership cannot be determined) and sets
`fallback_triggered`; this safety fallback does not require
`--global-config-fallback`. Its bounded helper closure follows ordinary static
imports/re-exports and literal CommonJS `require(...)` or
`require.resolve(...)` dependencies, retaining their edits and deletions as
owner triggers; computed or non-literal forms are not followed.

Resolved paths use `via: ["vitest-setup"]`. JSON may also contain the optional
aligned `via_details` array; its `{ "type": "vitest-setup", "field":
"setupFiles" | "globalSetup" }` entry names the setup field responsible for
that edge. `tests why` and `tests graph` expose the same structured `detail`.

`dotnet` plans require configured `.csproj` or `.sln` paths. They select
changed C# test files directly and select dependent C# tests through namespace
imports, type references, and `.csproj` `ProjectReference` edges. When native
tests are discoverable but the source/project change cannot be traced, the plan
falls back to the framework-scoped discovered tests and reports
`fallback_triggered`/`fallback_reason`. Command output uses `dotnet test
<project.csproj> --no-restore`. If no project target owns the selected test,
the fallback command is `dotnet test --no-restore`.

`swift` plans require `tests.swift.packages` config. They select changed Swift
tests directly and select dependent Swift tests through Swift graph edges and
HTTP route edges. When native tests are discoverable but the source/project
change cannot be traced, the plan falls back to the framework-scoped discovered
tests and reports `fallback_triggered`/`fallback_reason`.
