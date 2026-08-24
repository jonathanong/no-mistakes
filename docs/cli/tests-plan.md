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
no-mistakes tests plan python --changed-file app/users.py --format paths
no-mistakes tests plan go --changed-file pkg/ping.go --format commands
no-mistakes tests plan cargo --changed-file app/src/lib.rs --format commands
no-mistakes tests plan rails --changed-file app/models/user.rb --format paths
no-mistakes tests plan php --changed-file app/Http/Controllers/UserController.php --format commands
no-mistakes tests plan java --changed-file src/main/java/com/example/User.java --format commands
no-mistakes tests plan kotlin --changed-file src/main/kotlin/com/example/User.kt --format commands
no-mistakes tests plan elixir --changed-file lib/my_app/user.ex --format commands
no-mistakes tests plan dart --changed-file lib/user.dart --format commands
no-mistakes tests plan jest --changed-file src/value.ts --format commands
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
- `git-malformed-output` — a diff line exceeded the internal pathological-line
  bound, or Git returned a malformed/non-UTF-8 quoted path.

Node's `testsPlan()` rejects with the same stable code and message instead of
resolving to an empty plan.

JSON plans from the CLI keep snake_case keys. The Node `testsPlan()` /
`testsImpact()` APIs return camelCase only (`changedFiles`, `selectedTests`,
`executionTargets`, `fallbackTriggered`). `executionTargets` is the CI
contract: tests grouped by runner, config, project, and optional path-prefix
`name` (Swift packages). For configured framework plans, `--include-glob` /
`includeGlob` on `testsPlan()` scopes discovered tests before planning: limits,
fallback selection, group `remaining` counts, and execution targets all
describe only matching tests.

JSON plans include `changed_files`, the sorted, deduplicated, root-relative
inventory prepared by that invocation. It is present even when no tests are
selected and retains deleted paths plus both sides of detected renames and
copies. Non-JSON formats other than `explain` continue to render selected tests
only.

Manual `--changed-file` and `--changed-files` entries must remain within
`--root`, both lexically and after resolving an existing symlink. A symlink to
an in-root target keeps its lexical path in `changed_files`, while dependency
analysis follows the resolved target.

Key options: `--root`, `--config`, `--tsconfig`, `--environment`,
`--limit-percent`, `--limit-files`, `--global-config-fallback`,
`--include-comment`, `--include-glob`, `--format`, and `--json`.

`--format explain` renders a deterministic, human-readable plan: the normalized
changed-file inventory (including files that selected no tests), selected test
confidence, edge kinds, edge provenance when available, fallback state, and
warnings. Self-selected tests are rendered as node provenance rather than a
dangling edge. Use JSON when another program needs the same structured fields.

The configured `direct` group selects changed tests (`via: self`) and tests
one reverse import-family or same-directory `TestOf` edge away from a changed
file. Those 1-hop importers consume the environment budget before the
`dependencies` group, so a tight `limit.percent` / `limit.files` cannot drop a
unit test that directly imports the changed source in favor of a longer
markdown or resource path. Multi-hop imports and markdown/resource/route hops
stay in `dependencies`.

`--direct-test-owner` requires an explicit framework and performs a bounded
owner query instead of configured planning. It selects changed tests owned by
that framework plus only framework-owned tests connected by exactly one reverse
canonical graph edge, then attaches their execution targets. It ignores the
selected environment's groups, includes/excludes, limits, samples, and fallback
policy; `--entrypoint`, `--limit-percent`, `--limit-files`, and
`--global-config-fallback` therefore conflict with it. Direct-owner selection
is intentionally bounded to changed files and one reverse canonical graph edge;
use `no-mistakes tests impact --entrypoint <FILE>` for explicit entrypoint
traversal. Its result contains one `direct-test-owner` group and never reports
a fallback. It still reports canonical graph warnings for dynamic resource calls
in changed files, because they can make reverse owner selection incomplete.

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
global fallback. Resource impact is extension-agnostic: a Vitest test that
statically consumes a Markdown file or GitHub Actions workflow is selected when
that resource changes.

Framework selection filters test endpoints before graph traversal terminates.
`tests plan vitest` can terminate only at Vitest-owned tests, and
`tests plan playwright` only at Playwright-owned tests; canonical dependency and
resource edges between ordinary files remain available to both traversals.
`tests plan jest` uses explicit `tests.jest.configs` (or `tests.jest.projects`)
and parses static `testMatch` / `testRegex` literals. Empty `configs: []`
disables Jest discovery; there is no default-on `jest.config.*` scan. Jest
plans do not follow Vitest `setupFiles` / `globalSetup` edges.
Vitest `setupFiles` and `globalSetup` are included in the test dependency
graph automatically. A change to either configured module, or to a static
import/re-export reachable from one, selects only the tests owned by that
Vitest project. Inline projects inherit root setup fields only with
`extends: true`; otherwise (including the default and `extends: false`) their
own value applies, and `[]` clears it. A static string `extends` value on an
inline project (for example `extends: './vite.config.js'`) inherits that
config's `setupFiles` and `globalSetup`; local values are appended. A config
referenced as a string in `test.projects` is parsed as an independent config
and does not inherit the referencing config's setup fields.
Literal CommonJS project arrays are also supported as direct `require(...)`
entries or spreads. Named `projects` bindings destructured from a literal
require resolve static `module.exports.projects` and
`module.exports = { projects: [...] }` assignments; computed, dynamic, and
cyclic forms remain unsupported.
Direct requires only follow an exact `module.exports = [...]` assignment:
`module.exports.default` and `exports.default` remain object members, not
project arrays. Negated strings in supported imported arrays suppress matching
outer project strings before either config is parsed.
For supported inline objects, a nested `test` object owns `setupFiles` and
`globalSetup`; same-named outer fields are ignored regardless of direct or
static-spread declaration order.

Vitest workspace configs may export a project array directly or through
`defineWorkspace([...])`. With no `tests.vitest.configs`, root
`vitest.workspace.*` and `vitest.projects.*` files are discovered by default,
including JSON. JSON arrays accept static inline project objects and string
project paths/globs. Inline project `name` values may be a string or Vitest's
`{ "label", "color" }` object; its `label` is used for `--project`.
A project glob matches both visible files and visible
project folders. Folder matches select one conventional config directly inside
each matched project root, preferring `vitest.config.*` over `vite.config.*`;
nested roots require their own folder glob. Explicit project config-file globs
can select multiple files and recognize suffixes such as
`vitest.config.unit.ts` and
`vite.config.e2e.js`.
When a root workspace/project-array source is present, it is the default
discovery source instead of sibling `vitest.config.*`; list that config from
the workspace explicitly when it is also a project.
An exact folder project string remains a folder project rather than resolving
an `index` module. CommonJS workspace files may also use a direct literal
`module.exports = require('./projects.cjs')`; chained or dynamic requires stay
unsupported.
`defineWorkspace` is static through a named ESM import, an ESM namespace, or a
direct `require('vitest/config')` namespace; ESM defaults and CommonJS
`.default` members remain unsupported dynamic forms.

Setup values are extracted statically from string and array forms. Dynamic
expressions, unresolved literal modules, and unavailable static inline config
`extends` targets produce a JSON warning with the declaring config, field, and
project. When such a declaration is relevant, the
plan conservatively selects the affected owner scope (or the discovered Vitest
framework set when ownership cannot be determined) and sets
`fallback_triggered`; this safety fallback does not require
`--global-config-fallback`. Its bounded helper closure follows ordinary static
imports/re-exports and literal CommonJS `require(...)` or
`require.resolve(...)` dependencies, retaining their edits and deletions as
owner triggers. Static CommonJS setup bindings may use direct members or
destructured aliases, and helpers may expose named values through
`module.exports = { ... }`; computed or non-literal forms are not followed.

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
