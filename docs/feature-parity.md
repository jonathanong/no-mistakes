# Feature Parity

The opt-in [`package-json-export-target-prefix`](rules/package-json-export-target-prefix.md)
rule checks Node package export target syntax in both CLI and async Node reports.

The shell compatibility checks
[`bash-empty-array-nounset`](rules/bash-empty-array-nounset.md) and
[`zsh-readonly-parameter`](rules/zsh-readonly-parameter.md) share file selection,
source-line findings, and suppressions across Rust, CLI `check`, async Node
`check()`, and `analyzeProject()` check reports.

The opt-in `github-actions-no-required-pr-path-filters` reports the same PR trigger filter
keys and source lines across Rust, CLI `check`, and async Node check reports.
See [the rule](rules/github-actions-no-required-pr-path-filters.md).

GitHub Actions `workflow-topology-policy` supports opt-in
[`forbidReusableWorkflowConcurrency`](configuration/workflow-topology-policy-concurrency.md#forbidreusableworkflowconcurrency)
with identical workflow-level locations and suppression in Rust, CLI, and async
N-API check reports. Job-level locks remain allowed.

GitHub Actions `workflow-topology-policy` supports opt-in
[`forbiddenActivityTypes`](configuration/workflow-topology-policy-activity-types.md)
with the same explicit trigger matching, event filters, source locations, and
suppressions in Rust, CLI `check`, async Node `check()`, and `analyzeProject()`
check reports.

The same surfaces support opt-in
[`forbiddenEvents`](configuration/workflow-topology-policy-forbidden-events.md)
and
[`forbidSecretsInherit`](configuration/workflow-topology-policy-secrets-inherit.md).
Event bans cover scalar, sequence, and mapping `on` forms. The secrets ban
matches the exact `inherit` string on local and remote reusable-workflow calls.
Defaults stay off, and suppression stays on the reported key.

Native GitHub Actions checks include opt-in
[`github-actions-job-permissions`](rules/github-actions-job-permissions.md).
Rust, CLI `check`, async Node `check()` and `analyzeProject()` check reports use
the same workflow scope, inherited-write findings, unresolved policy, and
suppression handling.

[`github-actions-download-integrity`](rules/github-actions-download-integrity.md)
resolves literal step env URLs at max effort and supports opt-in Git-tracked
shell sources through `shellSourceInclude` and per-transfer checks through
`strictSequencing`. Rust, CLI `check`, async Node `check()`, and `analyzeProject()`
check reports share the selected sources, shell inference, findings, and
suppressions.

The opt-in [`github-actions-run-command-policy`](rules/github-actions-run-command-policy.md)
checks workflow and composite-action `run` scripts in Rust, CLI `check`, async
Node `check()`, and `analyzeProject()` check reports, sharing the same script
parser, argument policies, abstention advisories, and suppression handling.

The opt-in [`github-actions-workflow-script-paths`](rules/github-actions-workflow-script-paths.md)
checks that repository scripts run by path-filtered workflows (directly or via
local composite actions) fall inside each workflow's `paths` / `paths-ignore`
filters across Rust, CLI `check`, async Node `check()`, and `analyzeProject()`
check reports. [`github-actions-step-timeouts`](rules/github-actions-step-timeouts.md)
also supports per-action `actionCaps` and `onlyConfiguredActions` across all of
those entry points.

The opt-in [`dependabot-cooldown-policy`](rules/dependabot-cooldown-policy.md)
checks `.github/dependabot.yml` in Rust, CLI `check`, async Node `check()`, and
`analyzeProject()` check reports. These entry points share the same cooldown
floor, explicit PR limit check, exclusion globs, ecosystem exemptions, and
incomplete-input diagnostics.

The opt-in [`terraform-iam-no-undocumented-wildcards`](rules/terraform-iam-no-undocumented-wildcards.md)
checks native Terraform and OpenTofu source through Rust, CLI `check`, and
async Node `check()` or `analyzeProject()` check reports.

[`no-cross-package-relative-imports`](rules/no-cross-package-relative-imports.md) checks recorded
TS/JS imports against declared workspace ownership through Rust, CLI `check`,
and async Node `check()` and `analyzeProject()` check reports.

Playwright config `testMatch` / `testIgnore` accept static string globs, RegExp
literals, and mixed arrays in both CLI and Node analysis. Unsupported matchers
produce scoped aggregate-check warnings; see
[matcher recovery](configuration/tests.md#playwright-file-matchers).

The ESLint plugin provides opt-in, file-local test hygiene checks for
[fixed temporary paths](eslint-rules/test-no-fixed-temp-paths.md),
[typed enqueue fakes](eslint-rules/test-no-typed-enqueue-fake.md), and
[wall-time assertions](eslint-rules/test-no-wall-time-assertions.md). These
checks belong to ESLint/Oxlint; they do not add Rust CLI or native Node rules.

`no-mistakes` is a local, deterministic codebase-intelligence engine. TypeScript
and JavaScript are the reference implementation for the full product surface.
This page is the contract for bringing other languages and their key frameworks
up to that surface.

TS/JS `call-sites` resolves exported callable identities through namespace calls,
static lexical aliases, and barrels, excluding shadowed bindings. The `effects`
query and `query-reached-per-item` rule support opt-in module/export selectors with the same prepared call
resolver while retaining spelling-based string configuration. CLI, async Node,
and batched `analyzeProject` queries share this behavior.

Statically named top-level `const` CommonJS namespace, destructured, and member
bindings use the same canonical imported-call projection. Shadowed bindings stay
separate, and unsupported loads retain the
[`test-no-live-repository-scan`](rules/test-no-live-repository-scan.md) unresolved
fallback. That rule also recognizes direct `import.meta.glob` calls and `fs-extra`
enumerators, and distinguishes the shared temporary directory from an owned
`mkdtemp` result. A `fs/promises` `mkdtemp` promise that has not been awaited,
and the return value of callback `fs.mkdtemp`, are not that owned directory.
Awaiting an immutable `const` alias of that promise is the directory. Another
use of a mutable buffer result leaves the binding unproven.
Opt-in `analyzers` entries share this CLI and async Node check. A configured
command matches a direct executable, a `node_modules/.bin` basename, a
`.js`, `.mjs`, or `.cjs` path, a static `exec` or `execSync` shell string
(including a `cd <dir>` before `&&` or `;`, and a `NAME=value` prefix), a `spawn` or
`spawnSync` call with boolean `shell: true`, `fork`, and repeated wrapping by
`pnpm exec`, `pnpm dlx`, `npm exec`, `npx`, `bunx`, `yarn`, `yarn run`, or
`yarn dlx`, including `yarn --cwd` and a long value flag such as
`--reporter <value>`. The `cwd` proof is the
same owned `mkdtemp` or fixture-directory proof; a missing `cwd` is the live
working directory. A `cd <dir>` before unquoted `&&` or `;`, a pnpm `-C` or
`--dir`, or a yarn `--cwd` replaces that proof. `||` does not apply the `cd`.
Unquoted `&&`, `;`, and `||` separate simple commands. Each simple command is
matched, and an unresolved directory does not hide a later live scan. `cd .`
stays in the current directory, `cd /` is the filesystem root, and `cd ..`
leaves an unowned directory. A quoted argument stays one token. The executable matches on
quote-stripped text. `$` and a backtick stay literal inside single quotes. An
analyzer name inside a quoted assignment value, or `$` or a backtick in
unquoted text or double quotes, leaves the shell string unmatched. `cd` matches
on its quote-stripped text. A quoted `"&&"` is not a separator.
Potential Node-origin `module.require` and imported `createRequire` loads retain
unresolved-root findings through the same CLI and async Node checks; factory
and loader aliases do not grant a bounded-path proof.

TS/JS `unconstructed-error-class` uses canonical class ancestry for imported
bases, including conservatively resolved package type declarations and
tsconfig-visible ambient modules, including export-equals namespaces. Positive
non-Error class ancestry is skipped; unresolved or opaque ancestry follows the
shared unanalyzable policy. CLI `check`, async `check()` and `analyzeProject()`
check reports share this behavior without widening ordinary import/call queries.

Explicit Vitest and Playwright configurations expose optional declared
case/hook/fixture deadline evidence through Rust, CLI
`check --include-runner-config-deadlines`, async Node `check()` and
`analyzeProject()` check reports. The shared prepared owner retains config
failures, absent/known/unknown slots and source/inheritance provenance. Absence
does not supply SDK defaults, and this evidence does not prove runtime timeout
enforcement. See [declared runner evidence](cli/check.md#declared-runner-config-evidence)
and the [agent reference](../plugins/no-mistakes/skills/no-mistakes/references/tests.md).

TypeScript/JavaScript route coverage includes opt-in registered Vitest integration
sources for exact canonical Next.js page routes. These retain Vitest ownership and share
the canonical `route-test` graph, while selector and fetch coverage remain browser
coverage. See [integration route coverage](configuration/integration-route-coverage.md).

The ESLint/Oxlint plugin also provides the file-local `test-no-skips` rule for
Vitest and Playwright. It bans `skip`, `skipIf`, `runIf`, `only`, `todo`, and
`fixme` by default, with an explicit `allow` option. Same-file `.extend()`
fixture wrappers remain test bindings; `testModules` admits Playwright `test`
exports from named modules. ESLint `files` globs determine which test files
are checked; the rule does not discover tests or infer repository-wide scope.
This remains a plugin capability and adds no `no-mistakes` CLI or N-API
command. See [test-no-skips](eslint-rules/test-no-skips.md).

The file-local `playwright-require-interactive-test-id` plugin rule recognizes
Radix-style single-child slots on configured interactive components. `slotProps`
defaults to `["asChild"]`; literal parent selectors cover the direct child,
while ambiguous props or children remain checked. See
[interactive test IDs](eslint-rules/playwright-require-interactive-test-id.md).

The opt-in ESLint/Oxlint `vitest-timeout-cap` rule checks effective Vitest
configuration and test/hook timeout overrides in files selected by ESLint
`files` globs. Its defaults cap configuration timeouts at 5,000 ms and
individual overrides at 30,000 ms. Overrides above the default cap require an
adjacent, non-placeholder `no-mistakes-timeout-reason` comment. ESLint and Oxlint
use the same rule; unresolved values can be ignored or treated as findings.
It adds no `no-mistakes` CLI or N-API command. See
[Vitest timeout cap](eslint-rules/vitest-timeout-cap.md).

Next.js redirect and rewrite checks recover immutable tuple maps and template
destinations through the same Rust rule used by the CLI and async Node checks.
Partial dynamic construction reports incomplete extraction; configured route
roots retain their existing semantics.

A language or framework is supported when an agent can ask the same structural
questions it already asks of TS/JS, get deterministic structured output, and
do so without shelling out to `rg` for the graph itself.

The PostgreSQL query annotation rule traces straight-line TS/JS SQL helpers,
imported helpers, and callback forwarding through request-scoped prepared facts.
CLI `check`, Node `check()`, and batched `analyzeProject()` checks share findings
and the default `unanalyzableSql: report` policy; explicitly choose `ignore` for
opaque leading SQL. This annotation evidence does not imply complete SQL for
structural PostgreSQL rules.

`sql-template-strings` `.append()` chains preserve the leading annotation at
low effort and compose text for full SQL consumers, with interpolations treated
as value placeholders. Conditional appends use bounded breadth-first alternatives;
loops retain zero/one previews and an unknown remainder. These behaviors share
Rust, CLI and async Node evidence; see [append composition](postgres-query-annotation/append-composition.md).
Low annotation checks also prove fixed literal return heads in local helpers,
including concise arrow bodies and local export aliases, and one-hop source imports from the prepared inventory, without running the
annotation evaluator. This prefix proof does not establish complete SQL.
The same low proof recognizes a fresh native builder returned after exact
same-file helpers append through their second of two plain parameters; CLI and
async Node checks use the shared prepared facts and keep max behavior intact.
Low CLI and async Node checks also share a closed primitive helper-result proof:
one local const, one executor read, and complete literal return paths.
Opaque append suffixes preserve a prefix only for the trusted
`sql-template-strings` tag; other tags and their later reads remain unresolved.
An exact source helper may forward a proven head into a direct inline
configured executor callback, with each imported call checked separately.
The same prepared low proof also covers a source-proven same-file callback
helper and an async inline callback with an exact SQL parameter reference census.
They also accept finite conditional expressions whose every arm has a named
literal head, plus one returned local trusted builder with append-only uses
before its final return. Tagged arms and returned builders retain the existing
escape checks.
Function declarations with explicit early returns or a total final switch
also join identical named or missing heads. The switch default must throw
directly or call a stable same-file throw-only function. All return paths must
produce fresh trusted builders before callers may append to the result.
CLI and async Node checks also share the low-effort proof for a local primitive
`let` string with a continuation-safe head, discarded literal/template `+=`
writes, and only direct configured executor reads in the same function.
Conditional appends and template interpolations need no SQL evaluation;
captures, aliases, reassignment, and dynamic ownership gaps remain unresolved.

The low-effort prefix proof also accepts a captured local trusted builder when
all of its exact-binding uses preserve the named head, regardless of closure
execution order. CLI, async Node, and the Rust library share this evidence.
They also share bounded literal names in a direct `for-of` tuple loop, a
two-literal name ternary, a string-literal `for-of`, or a
closed same-file helper/select call chain. An unresolved import leaves the
ternary and string-list forms under `unanalyzable`. Only the first ordinary comment
template is substituted; SQL suffixes and tagged bind values are not composed.
The same low check follows an exact runtime namespace destructure through
unique explicit source exports and a barrel's imported default helper. It
reuses prepared module facts and resolver identity for CLI and async Node checks.
Captured append-only closures also preserve a trusted builder's head at its
own helper return when every exact-binding use is closed. Low and mixed-demand
checks share this proof without composing SQL variants.
The same APIs share bounded append-only helper composition across prepared
modules and named workspace re-exports. A discarded caller append chain may
consume a helper result only when all returns preserve the exact input builder.
Unique runtime named imports that are exported locally retain that effect proof;
namespace, type-only, ambiguous, and unresolved bindings do not. Fresh
conditional builders also retain their head through imported append effects
only after every required effect resolves.
Function-local fresh conditional builders and reassigned builders retain their
head only when every arm and assignment has a trusted literal annotation.
Append-only helper parameters also retain their fixed head through callbacks
whose only parameter uses are discarded static appends, regardless of when
those callbacks run.
For helper results passed directly to an executor, the prepared low proof can
follow one direct source import whose helper
returns an immutable append alias of its first fresh tagged argument. It
checks every parameter and alias reference and permits only an inert native
default after the first argument; extra caller objects remain unresolved.
CLI and async Node checks share the same inert later-parameter default proof
and exact closed-object gate for a defaulted helper destructure.

Opt-in `postgres-sql-fragment-escape` reports cross-file writes, rebinds, and
namespace escapes of exported trusted SQL const fragments. It reads prepared
import and value-flow facts, and recorded import targets when the canonical
graph was built. It does not change that graph or
`postgres-no-mutable-sql-registry`. When a trusted SQL tag is configured, a
file with an operational error, incomplete value flow, or unreadable source is
listed on the check `incomplete` field and does not drop findings from complete
files. Include and exclude select those gaps. Recorded project-file targets,
including path aliases and workspace packages, are followed exactly. Node
builtins and external packages are outside the search; a unique project ambient
declaration can identify an external specifier. Unresolved project imports retain
gap reporting.

Opt-in `postgres-no-mutable-sql-registry` and
`postgres-no-unbounded-sql-identifier` checks follow cross-file registry writes
(including re-export aliases, `export *` barrels, namespace imports, `delete`,
and `Object.defineProperty`) and unbounded identifier interpolations
(`process.env`, imported config, request parameters, query rows, and
non-finite wrappers) through prepared import, value-flow, and embedded-SQL
facts plus the canonical graph. At max effort, closed literal registries
(local or imported, until a member mutation), hyphen-stripped `randomUUID`
names imported from `node:crypto` or `crypto`, dominating closed-set throw
or return guards over local or imported literal arrays until the guarded binding
is written, configured `identifierValidators`, and compatible
`finiteDomainAssertions` are bounded. A configured validator proves its returned
value and its argument at later uses after the call completes, including an
initializer call. Its trusted contract may enforce an allow-list or identifier
grammar plus length limit. `identifierSources` declares trusted bounded scalar
returns for producers such as catalog lookup wrappers. Both options match either
the call-site module selector or canonical resolved origin through named or
`export *` barrels and npm re-exports, including workspace packages; helper bodies
are not inspected. Typed catalogs populated at module load can supply an
explicit trusted source. Complete max-effort scans report unused source entries
as `unused-allow-entry` at the configuration item. A catalog-element type alone
does not validate an incoming identifier; runtime guards also cover immutable
scalar aliases captured after validation, including executor and transaction
callbacks. Property guards validate only their exact member path. Static
metadata-array lookups followed by a throwing or returning presence guard prove
their literal string fields. Private module-local helpers
may return rows when every call result stays immutable and only reads known
fields; any escaping or mutating caller invalidates the shared metadata source.
Literal properties selected by a cross-module helper from a closed record and
module constants built by closed catalog filter/map/join chains are bounded
when their literal fields, callbacks, and uses remain closed. Mutations, escapes,
accessors, and open fields invalidate those proofs. Literal prefixes with
generated hex suffixes and UUID constructor helpers also qualify: the UUID
helper's prefix parameter must receive a literal call argument, and async
forwarding must be awaited. Open prefixes, fake UUID producers, mixed returns,
and recursive return cycles retain findings.
Configured SQL tag `.text` fragments and built-in `sql-template-strings`
default/`SQL` tags support immutable module constants, append chains, and
synchronous helper returns, including imported statements and factories used
through configured executor callback types. The built-in tags need no
configuration. Open raw append values and escaping
or mutated builders do not establish that proof. Literal-plus-numeric-placeholder
arrays joined with literal separators are also bounded. CLI `check`, Node `check()`,
and batched `analyzeProject()` share findings. Immutable array-destructured
query rows retain source provenance; comma expressions retain their final
operand's value provenance. A direct adjacent scalar overwrite can prove one
array-row member finite; the shared statement fact pass records this narrow
proof, while other mutations retain conservative provenance. Unknown provenance and HTTP
`.query` collisions stay silent. See
[mutable SQL registries](rules/postgres-no-mutable-sql-registry.md) and
[unbounded SQL identifiers](rules/postgres-no-unbounded-sql-identifier.md).

The shared low-effort SQL syntax proof resolves local executor type aliases,
`typeof` proven executor values (including an immutable `const`, `using`, or
`await using` factory binding), and configured factory type queries such as
`Awaited<ReturnType<typeof beginTransaction>>`. The identifier and transformed
UUID rules also recognize a direct executor handoff to a resolved typed helper
or an explicitly typed callback. SQL and opaque uses in the helper remain
independent obligations. CLI and async Node checks share these proofs; maximum
effort keeps its existing analysis. See
[executor bindings](postgres-facts/executor-bindings.md).

Opt-in [`postgres-no-test-state-mutation`](rules/postgres-no-test-state-mutation.md)
uses explicit test and helper globs to reject
executed PostgreSQL schema or server-state changes and `pg_sleep`. It shares
bounded literal fragment alternatives from exact Vitest `it.each` runtime rows,
and identifiers bounded by a local literal `Set` with a dominating throw guard,
and prepared TS/JS and SQL facts, across CLI `check`, async Node `check()`, and
`analyzeProject()` checks. SQL held only as assertion text or snapshot data,
and production migrations outside the declared test scope, stay out of this
rule. Fixed SQL switch arms with a source-visible, proven throwing default
are checked through the shared prepared helper projection in both interfaces.
Finite local builder assignments also retain their reachable alternatives
when a branch ends in a direct throw. Its shared PostgreSQL SQL parser accepts
`ANALYZE VERBOSE a, b` and per-relation columns such as `ANALYZE a(col), b`;
malformed lists and parenthesized `ANALYZE` option lists remain unanalyzable.
Destructured or bound methods selected by `executorNames` on configured imported
receivers retain their executor origin through local aliases in both interfaces.
Non-call runtime references and defaulted destructuring parameters follow the
existing unanalyzable policy at both effort levels.

At low effort, test-state SQL can use bounded source helpers, literal-return
switches with source-proven abrupt defaults, correlated strict predicates in
fresh builders, and unmodified local maps selected by inline Vitest literal
rows. Unsupported branches retain advisories rather than becoming clear.

The opt-in `postgres-json-relation-ids` check inspects selected migration SQL
and executed embedded SQL for JSON identifier keys without a same-row foreign
key, plus unique fingerprint columns. CLI `check`, async Node `check()`, and
`analyzeProject()` check reports share its catalog-backed analysis and
non-failing advisory results. Patterns are explicit; see the
[rule contract](rules/postgres-json-relation-ids.md).

Import-aware executed SQL extraction follows named private callables returned
from nested branches of exported factories through the same Rust and async Node
ESM/CJS API. Static callable wrappers and cyclic return references retain their
existing behavior; opaque returned expressions and unsupported bodies remain
unproven. Recovered modifying SQL still requires statement-level classification.

v1 is the Swift/.NET bar plus the named key feature for each stack: a module
graph, `tests plan`, and either HTTP routes or queues. SwiftPM and NuGet
dependency artifacts participate in semantic native test planning. Playwright, React,
Next.js fetches, call-sites, dead-exports, and applying every rule family to every language it
can (for example the PostgreSQL rules over SQL in Rust code, tracked in
[#4740](https://github.com/jongleberry/no-mistakes/issues/4740)) are later work. There are no
per-language CLIs and none are planned: one `no-mistakes` invocation analyzes every
configured language in one session with one graph, so a Next.js app that calls a Rust
endpoint is a single analysis. `dependents --relationship <lang>` and
`tests plan python|go|cargo|rails|php|java|kotlin|elixir|dart` are the stable
per-language surface when those stacks are configured.

`pom.xml`, Gradle lockfiles, `mix.lock`, and `pubspec.yaml` stay native fallback only.

## Current Status

| Domain                  | Module graph                                   | Test plan                                                                | HTTP routes                                                                            | Queues                                                    | Status                           |
| ----------------------- | ---------------------------------------------- | ------------------------------------------------------------------------ | -------------------------------------------------------------------------------------- | --------------------------------------------------------- | -------------------------------- |
| TypeScript / JavaScript | yes                                            | Vitest, Playwright, Jest                                                 | Express, Hono, Koa, Fastify, NestJS, Next.js, Remix file routes                        | BullMQ, glide-mq                                          | shipped (tRPC procedures opt-in) |
| Swift                   | `swift-import`, `swift-ref`, `swift-package`   | `tests plan swift`, semantic `Package.swift` / `Package.resolved` impact | no (client `http` edges only)                                                          | declined: no documented language-specific producer or worker         | shipped, narrower                |
| .NET / C#               | `dotnet-using`, `dotnet-ref`, `dotnet-project` | `tests plan dotnet`, semantic project / central NuGet / lock impact      | ASP.NET `MapGet` / `[HttpGet]` literals                                                | declined: no documented language-specific producer or worker         | shipped (v1 extractors + plan)   |
| Python, Django, Celery  | `python-import`, `python-ref`                  | `tests plan python`                                                      | Django `path(`, Flask, FastAPI                                                         | Celery `.delay(` / `@shared_task`                         | shipped (v1 extractors + plan)   |
| Go, Asynq               | `go-import`, `go-ref`                          | `tests plan go`                                                          | net/http, Chi, Gin, Echo, Fiber literals                                               | Asynq `NewTask` / `HandleFunc`                            | shipped (v1 extractors + plan)   |
| Kafka                   | n/a                                            | n/a                                                                      | n/a                                                                                    | static topic produce/consume                              | shipped (v1 extractors)          |
| Rust                    | `rust-use`, `rust-mod`                         | `tests plan cargo`                                                       | Axum, Actix, Rocket literals                                                           | declined: no Rust job facts                               | shipped (v1 extractors + plan)   |
| Ruby on Rails           | `ruby-require`, `ruby-ref`                     | `tests plan rails`                                                       | `routes.rb` `to:` / `resources`                                                        | Active Job `perform_later`, Sidekiq `perform_async`       | shipped (v1 extractors + plan)   |
| PHP                     | `php-use`, `php-package`                       | `tests plan php`                                                         | Laravel `Route::` / static resource and `apiResources` forms, or Symfony attribute/YAML | Laravel `::dispatch` / `ShouldQueue` or Symfony Messenger | shipped (v1 extractors + plan)   |
| Java, Spring            | `java-import`, `java-ref`                      | `tests plan java`                                                        | Spring `@RequestMapping` / `@GetMapping` literals                                      | declined: no documented language-specific producer or worker         | shipped (v1 extractors + plan)   |
| Kotlin, Spring          | `kotlin-import`, `kotlin-ref`                  | `tests plan kotlin`                                                      | Spring `@RequestMapping` / `@GetMapping` literals on `.kt`                             | declined: no documented language-specific producer or worker         | shipped (v1 extractors + plan)   |
| Elixir, Phoenix         | `elixir-import`, `elixir-ref`                  | `tests plan elixir`                                                      | Phoenix verb literals, literal `scope` prefixes, and static `resources`                | declined: no documented language-specific producer or worker         | shipped (v1 extractors + plan)   |
| Dart / Flutter          | `dart-import`, `dart-ref`                      | `tests plan dart`                                                        | no Dart server graph; client `http` edges to TS routes                                 | declined: no documented language-specific producer or worker         | shipped (v1 extractors + plan)   |

CI workflows and Terraform/OpenTofu are adjacent graph domains, not language
frontends. They stay available to every language once files are tracked.

PostgreSQL statement policy checks SQL files and opt-in JavaScript/TypeScript
executor calls through shared prepared facts, with CLI and async Node parity.
Static SQL returned by imported helpers (relative, `tsconfig` `paths` and barrel
imports, with string-literal arguments spliced into literal-identifier
templates) is recovered for every embedded-SQL rule and mapped to the helper's
original file and line, including assigned results and appended fragments.
An ordinary SQL template can also insert finite literal table names from a
same-file helper whose imported default throw is proven by prepared module
facts; tagged template substitutions remain binds.
Static runtime namespace reads such as `names.TABLE` and `names.default` also
recover exported string constants through prepared imports and re-exports;
computed, optional, method, type-only, and non-const reads remain unknown.
Tagged builder fragments accessed through namespaces stay unknown because the
exported object can mutate.
Named imports, factory handles, and typed executor parameters honor configured
scope; SQL-file selection retains `sqlInclude` semantics. The `ddl` group covers
29 statement kinds, including routines and database utility commands.
`bannedSettings` checks named settings in SET commands and literal `set_config`
calls, including recovered routine bodies and static EXECUTE. Opt-in
`bannedClauses` rejects explicit extension versions and cascading drops, including
`ALTER TABLE` column and constraint drops and `TRUNCATE ... CASCADE`, using the
same SQL-file and finite embedded-alternative scope. See
[statement policy](rules/postgres-sql-statement-policy.md).

PostgreSQL 17 query and `CREATE TABLE` compatibility applies to the same SQL
source facts used by the CLI and async Node checks. Supported target and table
clauses retain typed facts; complete syntax the AST cannot represent produces
the configured unanalyzable advisory. The PostgreSQL 17 regression suite's
combined column `STORAGE` and table `USING` clauses retain typed table facts.
The suite's index opclass options, quantified `VALUES`, XML table expressions,
`TABLESAMPLE`, CTE
`SEARCH`, and CTE-backed `MERGE` whole-row casts use that same path. See the
[schema diagnostic migration](migrations/postgres-schema-unanalyzable.md).

PostgreSQL SQL shape checks have CLI and
async Node parity. The opt-in `banned-function-call` SQL shape consumes shared AST
function-call facts in SQL files and parameterized embedded SQL, including
recoverable PL/pgSQL expressions and static EXECUTE. Configured names preserve
qualified and quoted identifier identity; columns and aliases are inert.

PostgreSQL expression facts expose recursive null tests and distinctness
predicates, bind parameters, typed temporal literals, and discriminated ordinary
literal values equally through Rust and the async CJS/ESM/native source API.
Conservative nullable spans and completeness remain independent of execution
policy and INSERT provenance.

Import-aware executed PostgreSQL extraction stringifies literal booleans in
finite joins, plain templates, and string concatenation through Rust and async
CJS/ESM APIs. Trusted-tag value binds and opaque coercion boundaries are preserved.

PostgreSQL migration recovery recognizes complete extension creation options
and boolean user-role attributes as unsupported-but-valid advisories by default.
`CREATE SCHEMA` inside procedural bodies and integer `FOR` expression ranges
are recovered, while malformed syntax stays visible. CLI and async Node check
reports share these facts and original diagnostic spelling; see the
[schema diagnostic policy](migrations/postgres-schema-unanalyzable.md).

The pure PostgreSQL source API is available through the async native Node
binding in CJS and ESM. Its typed CREATE INDEX facts preserve the `ON ONLY`
relation modifier in the Rust facts and Node declarations.

PostgreSQL bounded-statement checks share prepared SQL facts across the CLI and
async Node API. Temporary view lifetimes include cascading drops of physical
materialized views, qualified namesake separation, and transaction or savepoint
rollback. Materialized views retain their physical relation identity.

tRPC procedures are a TypeScript-only opt-in graph: configured
`projects.*.trpc.routers` globs plus static `router({ user: { get: procedure.query() } })`
and `trpc.user.get.query()` calls emit `trpc-call` / `trpc-procedure` edges
through `src/router.ts#procedure:user.get` virtual nodes. Request
`--relationship trpc`. Empty router lists disable the extractor; there is no
hardcoded `src/trpc`. Unfiltered `dependencies` and `--relationship all` omit
these edges. Do not reuse queue virtual nodes or `--relationship queue`.

Rust v1 is a language frontend for configured `tests.rust.packages`: `use
crate/super/self` and `pub` declarations emit `rust-use` / `rust-mod` edges.
`#[path]` mods emit `rust-mod`. Static Cargo `path =` deps and `tests/`
integration files emit `rust-package` from the crate root (not an n² clique).
The existing `rust-*` filesystem rules, `--test cargo` globs, and `ci` Cargo
binary edges remain. `tests plan cargo` now emits `cargo test -p` targets.
There is no `no-mistakes rust` CLI by design. Use `dependents --relationship rust`
and `tests plan cargo`.

## Canonical Feature Set

These are the features a new language must cover, or explicitly decline with a
documented limit. The TS/JS command is the reference behavior.

**Module graph.** `dependencies`, `dependents`, and `related` over typed
edges in the canonical `DepGraph`. TS/JS uses `import`, `type-import`,
`dynamic-import`, `conditional-dynamic-import`, `require`, `workspace`, and `package`. Swift and .NET
already show the non-TS pattern: language-specific edge kinds behind
`--relationship swift` or `--relationship dotnet`. `importers` stays the
fast TS/JS-only reverse static-import scan; it does not walk language
graphs. Use `dependents --relationship <lang>` for those.

**Symbols.** Named declarations and who references them. TS/JS has `symbols`,
`exports-of`, `dead-exports`, and `call-sites`. Swift and .NET collect
declarations and references into facts and project them as `*-ref` edges.

**Test impact.** `tests plan`, `tests why`, `tests impact`, and
`impacted-checks` must select native tests from changed files once the language
graph exists. Full-suite fallback remains explicit opt-in.

**HTTP routes.** `server routes`, `server edges`, `server related`, and
`server contracts` list configured TS/JS route definitions and static client
calls. Language v1 extractors emit `route` edges into `DepGraph` for Django,
Flask, FastAPI, Go HTTP, Rails, Laravel, Symfony, Rust Axum/Actix/Rocket,
ASP.NET `MapGet` / `[HttpGet]`, and Spring `@RequestMapping` / `@GetMapping`
in configured Java and Kotlin packages;
query those with
`dependents --relationship route`. `server routes|edges|related` also project
those language `RouteRef` facts into the existing server report. Do not invent
a second route graph.

**Queues.** `queues edges`, `queues related`, and `queues check` connect
TS/JS producers to virtual job nodes to workers. `queues.requiredJobOptions`
adds `missing-job-option` when a producer omits a configured key;
`queues.unboundedJobRetention` adds `unbounded-job-retention` when completed or
failed jobs are kept without a bound; `queues.jobStateAsRecord` adds advisory
`job-state-as-record` findings when ephemeral queue job state drives retries or
re-enqueues; all three are opt-in and leave default output unchanged. Celery, Asynq, Kafka, Active
Job, Sidekiq, Laravel, and Symfony Messenger emit the same `queue-enqueue` /
`queue-worker` edges into `DepGraph`; query those with
`dependents --relationship queue`. The dedicated `queues` commands project
those language edges into the existing report. They do not get private graph
shapes. Swift, .NET, Java, Kotlin, Elixir, Dart, and Rust frontends
record no language-specific queue jobs. The shared Kafka scanner still
checks files under configured globs for the literal shapes below.

Configured TypeScript queue enqueue wrappers support object-argument property
selectors, immutable returned-callable aliases with per-call options, and
fixed queues with local/imported const object job definitions. These share
producer/worker edges and option checks with positional factories; unresolved,
spread, and mutable definitions abstain. See [queue configuration](configuration/queues.md).

**HTTP clients.** Static client calls produce `http` edges to matching route
files, the same way TS `fetch` and Swift `Endpoint` literals do. Configured
Python (`requests` / `httpx`), Go (`net/http`), Rust (`reqwest`), Rails
(`Faraday`, `HTTParty`, `RestClient`, `Net::HTTP`), PHP (`Http::` and
`->request`), Java and Kotlin (`getForObject` / `.uri`), Elixir
(`HTTPoison` / `Req` / `Tesla`), and .NET (`GetAsync` / `HttpRequestMessage`)
literals do the same. `server edges` reports them as `client-call`. Computed,
concatenated, and interpolated URLs stay non-edges. Dart stays on `Uri.parse` /
`http.*` literals aimed at configured TypeScript backends.

**Lockfiles.** `lockfile diff` parses npm-family lockfiles and `poetry.lock`,
`uv.lock`, `Pipfile.lock`, `go.mod`, `Cargo.lock`, `Gemfile.lock`, and
`composer.lock`. The diff is package name, version, fingerprint, and
resolution kind. `go.sum` is not parsed; Go package-change impact reads
require directives from `go.mod`. `pom.xml`, Gradle lockfiles, `mix.lock`,
and `pubspec.yaml` stay native fallback only. Test planning also understands
SwiftPM `Package.resolved` pins and NuGet `packages.lock.json` (including
TFM/RID variants), project `PackageReference`, and central `PackageVersion`
changes.

**Checks, CLI, and N-API.** Every stable CLI capability needs an async N-API
equivalent, fixture-backed tests, and docs. CLI `--format json` emits compact
serde JSON. N-API JSON entrypoints take and return UTF-8 `Buffer`s; the Node
facade still accepts options objects. Language-specific check rules
belong in `no-mistakes check` when they are repository-wide, not file-local.

TS/JS-only product surfaces stay TS/JS-only: Playwright coverage, React
traits, Next.js fetches, RSC callers, and ESLint/Oxlint rules. A Django or
Rails app does not need React analysis to be considered at parity.

## How A Language Lands

Follow the Swift and .NET adapter shape, not a second analysis session.

1. Explicit config names the packages, modules, or apps to analyze. No
   repository-wide inference.
2. One request-scoped session still owns inventory and `SourceStore`. The
   language collector borrows those inputs.
3. A fact pass extracts imports, declarations, references, and domain
   occurrences (routes, enqueue sites, workers) from each matching file.
   Heuristic extractors that strip comments first, matching
   `codebase/swift` and `codebase/dotnet`, are the expected first
   implementation. String literals stay in the source used for route, queue,
   and topic facts; stripping strings would erase those identities. A full
   grammar is allowed later only if it stays in-process, deterministic, and
   one-pass.
4. Relationship analyzers emit typed edges into `DepGraph`. Commands and
   reports project those edges; they do not rebuild a private index.
5. Test discovery reads the language's explicit package/project config and
   emits `TestExecutionTarget` rows (`pytest`, `go test`, `cargo test`,
   `bin/rails test` / `rspec`, `phpunit` / `artisan test`, `mvn test [-f <package>/pom.xml] -Dtest=`,
   `gradle [-p <package>] test --tests`).
6. Ship CLI, N-API, `docs/cli/*`, `docs/graph-edges.md`,
   `docs/configuration/tests.md`, and fixtures in the same change.

Counterexample: a separate per-language command (for example `no-mistakes python`) that walks the tree, parses
files again, and builds a standalone import graph. That violates prepared
analysis ownership.

## .NET, ASP.NET

.NET already has the Swift-bar module graph and `tests plan dotnet`. HTTP v1
extracts literal ASP.NET minimal APIs and MVC attributes inside configured
`tests.dotnet.projects` / `tests.dotnet.solutions`.

Static `app.MapGet("/users", ListUsers)` / `MapPost` / `MapPut` / `MapPatch` /
`MapDelete` and `[HttpGet("/users")]` / `[HttpGet("users")]` (normalized to a
leading `/`) emit `route` edges from the registration file to the file that
declares the handler method. Handler names are the bare method ident after an
optional type qualifier (`UserHandlers.ListUsers` → `ListUsers`).

Computed paths, lambdas, `[HttpGet]` with no template, `[HttpGet(Name = …)]`,
`MapGroup` prefixes, and conventional `{controller}/{action}` routing produce
no edge. Same-file controller attributes still appear in `server routes`.

Static `client.GetAsync("/users")`, `PostAsync`, `PutAsync`, `PatchAsync`,
`DeleteAsync`, `GetFromJsonAsync`, `PostAsJsonAsync`, and
`new HttpRequestMessage(HttpMethod.Get, "/users")` literals emit `http` edges
to the file that already stores the matching route, and `client-call` edges in
`server edges`. Interpolated `$"..."` URLs stay non-edges.

```csharp
app.MapGet("/users", UserHandlers.ListUsers);
[HttpGet("/orders")]
public object GetOrders() => new object();
```

Configure projects through `tests.dotnet.projects` the same way `tests plan
dotnet` already does. Empty project lists disable the extractor; there is no
hardcoded `Controllers/` glob. Queues are declined: no documented static
producer or worker. ASP.NET route literals stay the HTTP surface. A method-name
pattern would invent a framework these docs do not name.

## Python, Django, Flask, FastAPI, Celery

Python support is the language frontend. Django, Flask, FastAPI, and Celery
are configured domain extractors on top of it.

| Feature          | TS/JS reference           | Python equivalent                                                                                |
| ---------------- | ------------------------- | ------------------------------------------------------------------------------------------------ |
| Module graph     | `import` / `require`      | `import`, `from … import`, relative `.` / `..` imports                                           |
| Package identity | `package.json` workspaces | configured package roots; `pyproject.toml` / `setup.cfg` names                                   |
| Symbols          | exports and importers     | module-level `def` / `class` and qualified references                                            |
| Tests            | `tests plan vitest`       | `tests plan python` over pytest / unittest files                                                 |
| HTTP routes      | Express / Hono / Koa      | Django URLconf → view, plus configured Flask / FastAPI decorator literals                        |
| HTTP clients     | `fetch` `http` edge       | `requests` / `httpx` static literals; computed URLs are non-edges                                 |
| Queues           | BullMQ / glide-mq         | Celery `@shared_task` / `@app.task`, `.delay(` / `.apply_async(`                                 |
| Lockfile         | pnpm / npm / yarn / bun   | `poetry.lock`, `uv.lock`, and `Pipfile.lock` (`lockfile diff`)                                      |

Configure package roots the way Swift configures `tests.swift.packages`. Route
and queue paths stay under `projects.*.routes` and `projects.*.queues`. Do not
hardcode `urls.py`, `tasks.py`, or `settings.py` locations.

Static forms produce edges:

```python
from app.users import views
from .models import User

@shared_task(name="mail.send_welcome")
def send_welcome(user_id: int) -> None: ...

send_welcome.delay(user_id)
```

```python
urlpatterns = [
    path("api/users/", views.user_list),
]
```

Dynamic forms do not:

```python
importlib.import_module(module_name)
celery_app.send_task(task_name, args=args)
path(prefix + "/users/", views.user_list)
@app.route(prefix + "/users")
@router.post(prefix + "/items")
```

Django settings, middleware, and model graphs are out of scope for the first
cut unless they are needed to resolve a static import or route.

## Go, Asynq, HTTP

Go support is the language frontend. Asynq is the queue extractor. Configured
`net/http`, Chi, Gin, Echo, and Fiber string-literal registrations emit
`RouteRef` edges.

| Feature          | TS/JS reference     | Go equivalent                                                                                                    |
| ---------------- | ------------------- | ---------------------------------------------------------------------------------------------------------------- |
| Module graph     | file imports        | `import` of local packages from configured `go.mod` modules                                                      |
| Package identity | workspace packages  | `go.mod` module path plus configured package directories                                                         |
| Symbols          | named exports       | exported (`Uppercase`) funcs/types and references                                                                |
| Tests            | `tests plan vitest` | `tests plan go` → `go test` in owning packages                                                                   |
| HTTP routes      | `server routes`     | configured `net/http`, Chi, Gin, Echo, or Fiber registrations                                                    |
| HTTP clients     | `fetch` `http` edge | `http.Get` / `http.NewRequest` static literals; computed URLs are non-edges                                      |
| Queues           | BullMQ job name     | Asynq `NewTask("mail:welcome", …)` / `HandleFunc("mail:welcome", …)`                                             |
| Lockfile         | npm-family          | `go.mod` require directives (`lockfile diff`; not `go.sum`)                                          |

Asynq task type strings are the virtual job identity, same as a BullMQ job
name. A producer file gets `queue-enqueue`; the handler file gets
`queue-worker`.

```go
client.Enqueue(asynq.NewTask("mail:welcome", payload), asynq.Queue("default"))
mux.HandleFunc("mail:welcome", HandleWelcome)
http.HandleFunc("/health", Health)
r.Get("/users", Users)
```

Computed task types and `http.Handle(pattern, …)` where `pattern` is not a
string literal produce no edge. `go/ast` in-process is acceptable; invoking the
`go` tool for analysis is not.

## Kafka

The Kafka scanner looks for the supported literal shapes in files selected
by the configured producer and consumer globs. It emits the existing
`queue-enqueue` and `queue-worker` edges. For example, a Java producer calling
`kafkaTemplate.send("mail.welcome", payload)` under an enqueue glob has a
queue edge.
The virtual job identity is `<cluster>:<topic>` when a configured logical
cluster or broker namespace is present and non-empty, and the bare topic
otherwise. The queue name falls back to `default` when no cluster is set; that
fallback is not part of the job identity. Topic names are cluster-scoped, so
two clients that reuse `mail.welcome` on different clusters stay distinct. A
consumer group id is edge metadata (or a downstream node), never part of that
identity.

```ts
producer.send({ topic: "mail.welcome", messages });
consumer.subscribe({ topic: "mail.welcome" });
```

```python
producer.send("mail.welcome", value=payload)
consumer.subscribe(["mail.welcome"])
```

Topic names built at runtime, regex subscriptions, and broker admin APIs are
out of scope. Configure producer and consumer file globs through
`projects.*.queues` the same way BullMQ enqueue/worker globs are configured.

## Rust

Rust v1 is a language frontend at the Swift/.NET bar: configured
`tests.rust.packages` emit `rust-use` / `rust-mod` / `rust-package` edges,
`tests plan cargo` selects crate tests, and static Axum / Actix / Rocket
registrations emit `route` edges. Language `symbols` / `call-sites`
and PostgreSQL rules over SQL in Rust code
([#4740](https://github.com/jongleberry/no-mistakes/issues/4740)) are later work.
`Cargo.lock` diffs participate in `lockfile diff`.

| Feature          | TS/JS reference     | Rust equivalent                                                                                                                              |
| ---------------- | ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| Module graph     | `import`            | `mod`, `use crate::…`, `use super::…`, path attrs                                                                                            |
| Package identity | workspace packages  | configured `Cargo.toml` packages and path deps                                                                                               |
| Symbols          | named exports       | `pub fn` / `pub struct` / `pub enum` and `use` paths                                                                                         |
| Tests            | `tests plan vitest` | `tests plan cargo` → `cargo test -p <pkg>` for sibling `tests.rs`; `cargo test -p <pkg> --test <name>` only for `tests/` integration targets |
| HTTP routes      | `server routes`     | configured Axum, Actix, or Rocket registrations                                                                                              |
| HTTP clients     | `fetch` `http` edge | `reqwest::get` / `reqwest::Client::new().get` static literals; computed URLs are non-edges                                                   |
| Queues           | BullMQ              | declined: no Rust job facts                                                                                                                  |
| Lockfile         | npm-family          | `Cargo.lock` name, version, and checksum (`lockfile diff`)                                                                                   |
| Checks           | `unique-exports`    | keep the existing `rust-*` filesystem rules                                                                                                  |

The Rust fact pass records no language-specific queue jobs. Rust files
selected by a queue glob are still eligible for the shared Kafka scanner.

`ci` remains the narrow workflow-file → Rust-binary Cargo edge. Do not overload
it with `use`/`mod` edges. `workflow` already resolves supported Cargo
`run:` lines.

Static Axum `.route("/users", get(list_users))`, Actix
`web::resource("/ready").route(web::get().to(ready))`, and Actix/Rocket
`#[get("/health")]` attributes emit `route` edges. Handler names are the bare
function ident. Computed paths, `#[get(prefix)]`, and chained
`.route("/x", get(a).post(b))` produce no edge.

```rust
.route("/users", get(list_users))
#[get("/health")]
pub async fn health() {}
```

Inline `#[cfg(test)]` modules are banned by `rust-no-inline-tests` in this
repo; discovery should prefer `tests/**/*.rs` and sibling `tests.rs` files.

## Ruby on Rails

Rails support is Ruby module facts plus configured route, Active Job, and
Sidekiq extractors.

| Feature          | TS/JS reference     | Rails equivalent                                                                                        |
| ---------------- | ------------------- | ------------------------------------------------------------------------------------------------------- |
| Module graph     | `import`            | `require`, `require_relative`, Zeitwerk-constant references inside configured app roots                 |
| Package identity | workspace packages  | configured engine/app roots and `Gemfile` path gems                                                     |
| Tests            | `tests plan vitest` | `tests plan rails` over Minitest / RSpec files                                                          |
| HTTP routes      | `server routes`     | configured `config/routes.rb` (and engine routes) → controller#action, including bare `resources :name` |
| HTTP clients     | `fetch` `http` edge | `Faraday` / `HTTParty` / `RestClient` / `Net::HTTP.get` static literals; computed URLs are non-edges  |
| Queues           | BullMQ              | Active Job `SomeJob.perform_later` or Sidekiq `SomeWorker.perform_async` → job class                    |
| Lockfile         | npm-family          | `Gemfile.lock` specs (`lockfile diff`)                                                                  |

Zeitwerk inference is heuristic and must stay inside configured roots. Do not
scan the whole repository for `app/models`. Dynamic `constantize`,
`send(:"#{name}_path")`, and `perform_later` / `perform_async` on a computed
job class produce no edge. `only:` / `except:`, singular `resource`, and
namespaced `resources` produce no extra route edges. Bare `resources :users`
and `resources "users"` expand to index/show/create/update/destroy.

```ruby
get "/api/users", to: "users#index"
resources :users
WelcomeJob.perform_later(user)
MailWorker.perform_async(user)
```

## PHP

PHP support is Composer/PSR-4 facts plus one configured framework extractor.
Set `tests.php.framework` to `laravel` or `symfony`. Do not infer the
framework from files, and do not enable both extractors from a missing value.

| Feature          | TS/JS reference     | PHP equivalent                                                                                                |
| ---------------- | ------------------- | ------------------------------------------------------------------------------------------------------------- |
| Module graph     | `import`            | `use`, `require`/`include` of local files, PSR-4 from configured `composer.json`                              |
| Package identity | workspace packages  | configured Composer packages / path repositories                                                              |
| Tests            | `tests plan vitest` | `tests plan php` over PHPUnit / Pest files                                                                    |
| HTTP routes      | `server routes`     | configured Laravel `Route::` / static `Route::resource` / `Route::apiResource` / `Route::apiResources` or Symfony attribute/YAML routes |
| HTTP clients     | `fetch` `http` edge | Laravel `Http::get` or `->request("GET", ...)` static literals; computed URLs are non-edges                  |
| Queues           | BullMQ              | Laravel `SomeJob::dispatch()` / `ShouldQueue`, or Symfony Messenger handlers                                  |
| Lockfile         | npm-family          | `composer.lock` packages (`lockfile diff`)                                                              |

```php
Route::get('/api/users', [UserController::class, 'index']);
Route::resource('users', UserController::class);
Route::apiResource('accounts', UserController::class);
Route::resource('articles', UserController::class)->only(['show']);
Route::apiResources(['bulk' => UserController::class]);
SomeJob::dispatch($user);
#[Route('/health', methods: ['GET'])]
class HealthController {}
$bus->dispatch(new WelcomeMessage());
```

`__DIR__ . '/' . $name`, `app($abstract)`, `#[Route($prefix . '/users')]`,
and `$bus->dispatch($message)` lookups are non-edges. Static
`Route::resource` / `Route::apiResource` forms with an `only` or `except`
list (an options array or `->only([...])` / `->except([...])`, including
after `->names([...])`), nested dotted names such as `photos.comments`,
named `name:` / `controller:` arguments, and `Route::apiResources` maps
emit one route fact per remaining standard action and omit excluded actions.
`Route::apiResource` omits `create` and `edit`. Computed prefixes, computed
names, nested routes using `->shallow()`, and `__DIR__` concatenation stay
non-edges. A call that sets both
`only` and `except`, or names an action outside that call's set, stays a
non-edge. Missing `tests.php.framework` still enables neither Laravel nor
Symfony extractors.

## Java, Spring

Java support is a language frontend for configured `tests.java.packages`.
Empty lists disable the extractor; there is no hardcoded `src/main/java`.
Exact `import com.example.User;` statements emit `java-import`. Star imports
and `import static` are non-edges. Class/interface/enum/record names plus
capitalized identifiers emit `java-ref`.

Spring HTTP v1 combines a class `@RequestMapping("/api")` prefix with method
`@GetMapping("/users")` / `@PostMapping` / `@PutMapping` / `@PatchMapping` /
`@DeleteMapping` / `@RequestMapping` literals. Absolute method paths still
join the class prefix. Computed paths, empty mappings, and `{id}` client
wildcard translation are non-edges. Same-file controller methods do not emit
`RouteRef` (self-edges are skipped) but still appear in `server routes`.

| Feature          | TS/JS reference     | Java equivalent                                                                                             |
| ---------------- | ------------------- | ----------------------------------------------------------------------------------------------------------- |
| Module graph     | `import`            | exact `import com.example.User;`                                                                            |
| Package identity | workspace packages  | configured `tests.java.packages`                                                                            |
| Tests            | `tests plan vitest` | `tests plan java` over `*Test.java` / `*Tests.java` / `*IT.java`; `mvn test [-f <package>/pom.xml] -Dtest=` |
| HTTP routes      | `server routes`     | Spring `@RequestMapping` + `@GetMapping` literals                                                           |
| HTTP clients     | `fetch` `http` edge | `getForObject` / `.uri` static literals; computed URLs are non-edges                         |
| Queues           | BullMQ              | declined: no documented language-specific producer or worker                                                                           |
| Lockfile         | npm-family          | later (`pom.xml` native fallback only)                                                                      |

```java
package com.example;
import com.example.User;

@RequestMapping("/api")
public class Users {
  @GetMapping("/users")
  public Object listUsers() { return User.list(); }
}
```

`@GetMapping(PREFIX)`, `@GetMapping`, extra mapping attributes after the path,
intervening non-annotation noise that breaks the method matcher, class-only
`@RequestMapping` without a method mapping, same-package type refs without an
explicit `import`, and `import com.example.*` are non-edges. The Java queue cell is
declined because no language-specific producer or worker is documented.
The shared Kafka scanner still recognizes supported calls under configured
globs.

## Kotlin, Spring

Kotlin support is a language frontend for configured `tests.kotlin.packages`.
Empty lists disable the extractor; there is no Gradle inference and the list
is not folded into `tests.java`. Exact `import com.example.User` statements
emit `kotlin-import`. Star imports are non-edges. Optional `as` aliases still
record the original fully-qualified class name. Class/interface/object names plus capitalized
identifiers emit `kotlin-ref`.

Spring HTTP v1 reuses the Java mapping literals on `.kt` files, matching
`fun listUsers()` handlers after `@GetMapping("/users")`.

| Feature          | TS/JS reference     | Kotlin equivalent                                                                                  |
| ---------------- | ------------------- | -------------------------------------------------------------------------------------------------- |
| Module graph     | `import`            | exact `import com.example.User`                                                                    |
| Package identity | workspace packages  | configured `tests.kotlin.packages`                                                                 |
| Tests            | `tests plan vitest` | `tests plan kotlin` over `*Test.kt` / `*Tests.kt` / `*IT.kt`; `gradle [-p <package>] test --tests` |
| HTTP routes      | `server routes`     | Spring `@RequestMapping` + `@GetMapping` literals on `.kt`                                         |
| HTTP clients     | `fetch` `http` edge | `getForObject` / `.uri` static literals; computed URLs are non-edges                |
| Queues           | BullMQ              | declined: no documented language-specific producer or worker                                                                  |
| Lockfile         | npm-family          | later (`build.gradle.kts` native fallback only)                                                    |

```kotlin
package com.example
import com.example.User

@RequestMapping("/api")
class Users {
  @GetMapping("/users")
  fun listUsers(): Any? = User.list()
}
```

`@GetMapping(PREFIX)`, `@GetMapping`, extra mapping attributes after the path,
same-package type refs without an explicit `import`, `import com.example.*`,
top-level functions/properties, extra types in the same file, multi-class
`@RequestMapping` prefixes, and annotation examples inside raw strings are
non-edges. The Kotlin queue cell is declined because no language-specific producer or
worker is documented. Native fallback is `build.gradle` / `build.gradle.kts` plus
non-test `.kt` files; `settings.gradle*` is not a trigger. `--tests` uses the
file stem, matching Java `-Dtest`.

## Elixir, Phoenix

Elixir support is a language frontend for configured `tests.elixir.apps`.
Empty lists disable the extractor; there is no `mix.exs` inference.
Exact `alias`/`import`/`use MyApp.User` statements and brace aliases
`alias MyApp.{User, Role}` emit `elixir-import`. Nested braces
`alias MyApp.{Foo.{Bar}}` and wildcard imports are non-edges.
Module names plus capitalized identifiers emit `elixir-ref`. Same-package
refs without an `alias`/`import`/`use` are non-edges.

Phoenix HTTP v1 matches literal `get "/users", Controller, :index` (and
`post`/`put`/`patch`/`delete`) registrations, joins literal `scope "/api"`
prefixes onto nested verb and `resources` paths, and expands a static
`resources "/users", UserController` macro into one route fact per default
action (`index`, `create`, `new`, `show`, `edit`, `update`, `delete`).
Computed `scope path` blocks stay non-edges, including a `resources` macro
nested inside one. A `resources` call with options such as `only:` stays a
non-edge.

| Feature          | TS/JS reference     | Elixir equivalent                                                                                                            |
| ---------------- | ------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Module graph     | `import`            | exact `alias`/`import`/`use MyApp.User` and brace aliases `alias MyApp.{User, Role}`                                         |
| Package identity | workspace packages  | configured `tests.elixir.apps`                                                                                               |
| Tests            | `tests plan vitest` | `tests plan elixir` over `*_test.exs`; `mix test <path>` from the Mix project root (umbrella child paths stay repo-relative) |
| HTTP routes      | `server routes`     | Phoenix verb literals, literal `scope` prefixes, and static `resources` macros                                                |
| HTTP clients     | `fetch` `http` edge | `HTTPoison` / `Req` / `Tesla` static literals; computed URLs are non-edges                                                       |
| Queues           | BullMQ              | declined: no documented language-specific producer or worker                                                                            |
| Lockfile         | npm-family          | later (`mix.exs` native fallback only)                                                                                       |

```elixir
defmodule MyApp.App do
  alias MyApp.{User, Role}
end

defmodule MyAppWeb.Router do
  get "/users", MyAppWeb.UserController, :index
  post "/users", MyAppWeb.UserController, :create
  scope "/api" do
    get "/items", MyAppWeb.UserController, :index
  end
  resources "/accounts", MyAppWeb.UserController
end
```

Nested brace aliases, wildcard imports, same-package refs without
`alias`/`import`/`use`, computed `scope path` blocks (including a nested
`resources` macro), `resources` calls with options, and `mix.lock` /
`config.exs` native fallback are non-edges. The Elixir queue cell is declined because no
language-specific producer or worker is documented. Native fallback is
`mix.exs` plus non-test `.ex` files under configured apps. Any `.ex`/`.exs`
under `/test/` is non-production. `*_test.exs` is the test suffix.

## Dart, Flutter

Dart support is a language frontend for configured `tests.dart.packages`.
Empty lists disable the extractor; there is no pubspec inference of the
package list. Exact `import` / `export` / `part` / `part of` URIs emit
`dart-import`. `dart:` and `*` URIs are non-edges. Brace-free `package:app/user.dart`
imports resolve against `pubspec.yaml` `name:` plus `lib/`. Relative URIs
resolve then remap to that package URI when they stay under `lib/`.
Class/mixin/enum/extension/typedef names plus capitalized identifiers emit
`dart-ref`.

HTTP v1 is client-only: literal `Uri.parse("/api/...")` and
`http.get("/api/...")` (post/put/patch/delete) emit `http` edges to matching
configured TypeScript backend routes. Hosted URLs keep the path after the
host. There is no Dart server graph. `resources`-style and computed URIs are
non-edges.

| Feature          | TS/JS reference                 | Dart equivalent                                                                                                  |
| ---------------- | ------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| Module graph     | `import`                        | exact `import 'package:app/user.dart'` / relative `lib/` URIs                                                    |
| Package identity | workspace packages              | configured `tests.dart.packages` + `pubspec.yaml` `name:`                                                        |
| Tests            | `tests plan vitest`             | `tests plan dart` over `*_test.dart`; `dart test <path>` or `dart pub --directory <package> run test <rel-path>` |
| HTTP routes      | Next.js / Express client `http` | client `Uri.parse` / `http.*` literals to TS backend routes                                                      |
| Queues           | BullMQ                          | declined: no documented language-specific producer or worker                                                            |
| Lockfile         | npm-family                      | later (`pubspec.yaml` native fallback only)                                                                      |

```dart
import 'package:app/user.dart';
import 'package:http/http.dart' as http;

Future<void> loadUsers() async {
  await http.get(Uri.parse("/api/users"));
  User.list();
}
```

`dart:async`, `import 'package:app/*'`, computed `Uri.parse(prefix + "/users")`,
and files outside configured packages are non-edges. Queues are declined: no
documented static producer or worker. There is no Dart server graph to attach
one to. Native fallback is
`pubspec.yaml` plus non-test `.dart` files; any `.dart` under `/test/` is
non-production. v1 emits `dart test`, not `flutter test`.

## Shared Domain Rules

The CLI and async Node check APIs share the opt-in [`react-no-nested-buttons-and-links`](rules/react-no-nested-buttons-and-links.md) rule for native buttons and links in selected JSX and TSX files.
The CLI and async Node check APIs share the opt-in
[`swift-no-unguarded-darwin-import`](rules/swift-no-unguarded-darwin-import.md)
rule for portable Swift files.

Language route visitors emit `route` edges. HTTP callers emit `http`.
Language queue visitors and the shared Kafka scanner emit
`queue-enqueue` and `queue-worker`. `server-route` and
`client-call` are server-report kinds, not language graph edges. A Celery
producer in Python and an Asynq producer in Go must be traversable with
`--relationship queue`.

Relationship filters for the language graph itself follow Swift/.NET:

- `python` — Python import and reference edges
- `go` — Go import and reference edges
- `rust` — Rust `use`/`mod` and path-dep edges
- `ruby` — Ruby require and configured constant edges
- `php` — PHP `use` / Composer edges
- `java` — Java import and reference edges
- `kotlin` — Kotlin import and reference edges
- `elixir` — Elixir import and reference edges
- `dart` — Dart import and reference edges

Additive language flags must not change existing TS/JS report fields. When a
broader resolver catalog is needed (for example tests that live outside the
app package), add a fixture-backed parity test that compares baseline fields
with the flag off and on.

## Configuration Shape

These knobs are shipped, explicit, and scoped. Empty package/module lists
disable the corresponding language frontend; no repository-wide fallback is
inferred. See [queue configuration](configuration/queues.md) for the queue
factory and project-glob details.

```yaml
projects:
  api:
    type: server
    root: backend
    routes: ["config/urls.py", "routes/**/*.py"]
    queues:
      cluster: orders
      enqueues: ["app/**/tasks.py"]
      workers: ["app/**/tasks.py", "workers/**/*.py"]
    trpc:
      routers: ["src/trpc/**/*.ts"]
queues:
  factories: [createQueue] # or { queue, enqueue, worker } argument indexes
tests:
  python:
    packages:
      - backend
  go:
    modules:
      - services/worker
  rust:
    packages:
      - crates/api
  rails:
    apps:
      - apps/web
  php:
    framework: laravel
    apps:
      - services/api
  java:
    packages:
      - services/api
  kotlin:
    packages:
      - services/api
  elixir:
    apps:
      - apps/web
  dart:
    packages:
      - apps/mobile
```

Counterexample: defaulting to “every `urls.py`, every `go.mod`, every Rails
`app/`” when the lists are omitted. Full-suite and repository-wide scans stay
opt-in.

## Agent Fallback

v1 module graphs and `tests plan <lang>` are shipped for configured Python,
Go, Rust, Rails, PHP, Java, Kotlin, Elixir, and Dart packages. Named route
extractors are shipped where the status table names them. Named queue
extractors are shipped for TypeScript, Python, Go, Kafka, Rails, and PHP.
Swift, .NET, Java, Kotlin, Elixir, Dart, and Rust have declined
language-specific queue cells. Those files remain eligible for shared Kafka
literal scanning under configured globs.
One invocation covers every configured language. Use `dependents --relationship <lang|route|queue|http>` and
`tests plan python|go|cargo|rails|php|java|kotlin|elixir|dart` for the shipped
questions instead of `rg`.

Configured language clients can match static root-relative paths and literal
HTTP(S) URLs on loopback hosts. Use `rg` for client calls with other hosts or
call shapes outside the documented matchers.

Queue cells that say `declined` are closed. Do not invent a framework for them.

Keep using `rg` for holes the status table still marks `no` or later:
Laravel resource routes with computed options or arguments, Phoenix resource
routes with options, Kafka calls outside the documented literal shapes, language
`symbols`/`call-sites`, and rule families that apply only to TS/JS or `.sql`
([#4740](https://github.com/jongleberry/no-mistakes/issues/4740)).

See [Architecture](architecture.md) for the one-pass session rules,
[Graph edges](graph-edges.md) for the current edge kinds, and
[Tests and selectors](configuration/tests.md) for how Swift and .NET
already require explicit package/project lists.

PostgreSQL schema catalog generation (complete and ordering coverage) has Rust library, CLI and async Node/N-API parity, including opt-in search-path schema evidence; see [`postgres catalog`](cli/postgres.md).
PostgreSQL source parsing accepts `CREATE [OR REPLACE] RECURSIVE VIEW` with explicit column lists through Rust and async Node ESM/CJS APIs, emitting typed `createView` facts. Strict and lenient statement analysis preserve the implicit recursive CTE scope; see [PostgreSQL source API](postgres-source-api.md).

PostgreSQL source parsing accepts COMMENT ON FUNCTION signatures, ALTER INDEX
ATTACH PARTITION, and plain newline-separated string continuations through the
Rust library and async Node ESM/CJS APIs, with typed metadata, original spans,
and malformed-source recovery; see [PostgreSQL source API](postgres-source-api.md).

PostgreSQL bounded-statement analysis recognizes `TABLE ONLY name` query arms and retains quoted relation identity across strict and lenient parsing.
Executed PostgreSQL SQL checks accept a comma-separated `ANALYZE` relation list as complete SQL; malformed lists still produce an unanalyzable result.
It also recognizes `INSERT INTO target TABLE ONLY source`, including target aliases and identity overrides, while retaining the original source-token locations.
It also analyzes standalone `TABLE name` and `TABLE ONLY name` statements, including an explicit inheritance `*` suffix, with the same catalog and temporary-relation semantics as a corresponding `SELECT * FROM name`.
Three-arm set operations with a middle `TABLE` and a following `SELECT` retain all arms and quoted relation identities.
Supported `TABLE` set-operation arms also project their exact relation identity and source line into shared Select facts for `postgres-explicit-columns` and `postgres-required-predicates`.
`EXPLAIN TABLE name` parses as a plan without execution, while `EXPLAIN ANALYZE TABLE name` contributes executed query facts with the same relation identity.
Lenient PostgreSQL recovery retains exact TABLE-arm spelling inside DO bodies and reconstructed SQL fragments.
Parenthesized PostgreSQL set operations retain a trailing LIMIT after an unqualified TABLE arm.

Parenthesized PostgreSQL `TABLE relation` scalar subqueries retain quoted and qualified identity in view dependencies, including temporary-view and permanent-intermediary cascades; visible CTE aliases are excluded.
Recursive `WITH` bodies can also start with `TABLE` or use it as a set-operation arm. Forward CTE dependencies resolve before projection, and one quoted name containing a dot remains distinct from a schema-qualified name.

Selected TS/JS module binding and import facts are available through the async
`analyzeTypeScriptModules` Node API; see [Node API](node-api.md#selected-typescript-and-javascript-module-facts).

PostgreSQL pure-source Rust and async Node APIs project INSERT/UPDATE/DELETE/MERGE
CTE bodies through `query.nestedStatements`, including source provenance, typed
RETURNING/actions and explicit incomplete children. Top-level INSERT facts
include `returning`. A RETURNING data-modifying CTE insert includes the same
`columnSources` lineage as the direct statement. CJS and ESM use the same
native projection. This surface reports syntax without executing SQL.

PostgreSQL source wrapper facts have Rust and asynchronous Node ESM/CJS parity:
EXPLAIN distinguishes analysis execution, PREPARE stays non-executing, and SQL
BEGIN ATOMIC function declarations retain non-executing child source occurrences.
Unsupported body or option projections are explicitly incomplete; see
[statement wrappers](postgres-source-api.md#statement-wrappers-and-execution-context).

Literal PL/pgSQL `EXECUTE` nested SQL facts have Rust/native and asynchronous
Node ESM/CJS parity, including decoded child spans and original wrapper
provenance. Dynamic expressions remain explicitly unsupported; see the
[PostgreSQL source API](postgres-source-api.md).

The shared Rust/native parser and asynchronous Node APIs also recognize the
selected PostgreSQL 17 DDL forms in the
[schema migration guide](migrations/postgres-schema-unanalyzable.md), including
default-privilege revocations, publication options, and metadata from the saved
statement-family survey. Recognized unsupported metadata retains diagnostics
and advises by default across the eleven schema rules; malformed top-level SQL
continues to report. Explicit policies and report shapes are unchanged.

Procedural `DO` occurrence kinds (`utility`, `controlFlow`, `dml`,
`dynamicExecute`, `unknown`) have the same Rust/native and asynchronous Node
ESM/CJS projection. Nested `dml` is source text, not execution.
Procedural completeness requires supported static SQL to be reachable through
typed statements. Unprojected utilities inside `FOR`, `WHILE`, `FOREACH`, and
bare `LOOP`, including nested loops and `IF`/`ELSE` combinations, produce
diagnostics and keep the block incomplete. Typed conditional branches and
recognized utilities represented as typed `other` statements retain coverage.
Occurrence classification is additive, with original spans and source order.
Dynamic `EXECUTE` stays fail-closed. Nested literal `EXECUTE` commands share that
walker's 64-level budget and fail closed when it is exhausted. A wholly literal
`FOR ... IN EXECUTE` operand is classified as `dml` or `utility`, including a
parsed `SELECT`. A qualified name such as `public.execute(...)` stays
`dynamicExecute` rather than static DML parsed from the argument. An IF, ELSIF,
or conditional-loop header with no condition keeps the block incomplete. That
empty header reports that the procedural condition is missing, at the header. An
opening label is one identifier; PL/pgSQL reserved words such as unquoted
`BEGIN` are not labels, while unquoted `SELECT` is allowed. Labels compare
after unquoted case folding and PostgreSQL's 63-byte UTF-8 identifier
truncation; quoted and unquoted forms can match when their stored names agree.
A closing label that does not match keeps the block incomplete and reports
`unknown` at the mismatched label, in source order. Multiple mismatches retain
their enclosing blocks through one ordered merge without repeated sibling
scans. A label prefix on a wholly literal command stays `unknown`. An opening
label is omitted with the walker-only block it names.

PostgreSQL INSERT source facts have Rust and asynchronous Node ESM/CJS parity
for composite assignment syntax. Function expressions expose `derived` provenance;
unary roots expose typed operands, and syntax completeness is independent of
atomic lineage, including COALESCE and
GREATEST over target and EXCLUDED references. Unsupported expression projections
remain explicitly incomplete; see [INSERT source facts](postgres-source-api.md).

PostgreSQL expression facts expose ordered typed recursive children, and INSERT
facts map explicit target columns to VALUES rows and SELECT/set-operation
branches in Rust and asynchronous Node ESM/CJS APIs. Unsupported or ambiguous
source projections carry typed reasons; nullable spans do not guess wrapper
boundaries. See [PostgreSQL source facts](postgres-source-api.md).

PostgreSQL pure-source Rust, native, and asynchronous Node ESM/CJS APIs accept
ON CONFLICT expression arbiters with operator classes and mixed array/composite
assignment targets, retaining
ordered expressions and original source spans. Malformed targets diagnose while
neighboring statements survive; these facts do not resolve catalog indexes or
replay safety. See [PostgreSQL source API](postgres-source-api.md).

The extended conflict grammar applies to prepared outer INSERT clauses, including
EXPLAIN/PREPARE wrappers. A column-target `WHERE` inside a data-modifying CTE
INSERT is projected, including `DO NOTHING`, `DO UPDATE`, and `RETURNING`, and
the same predicate is kept when that CTE is the child of an EXPLAIN or PREPARE
wrapper.
Expression arbiters and subscripted assignments inside data-modifying CTE
INSERTs still diagnose through the upstream grammar; neighboring statements
remain available. Exact closing-delimiter spans apply
to the prepared conflict arbiters, assignment targets/RHS, and target/action
predicates; AST-only nested conditional/CTE projections retain existing span limits.
Operator classes expose optional ordered typed `parameters` (name and value),
including their full class source span. Leading composite fields enter the
ordered assignment indirection after its base column.

PostgreSQL CREATE/ALTER constraint source spans have Rust/native and asynchronous
Node CJS/ESM parity. Supported inline and table constraints retain exact source
ranges; unavailable boundaries are null rather than inferred from formatted SQL.
See [constraint source spans](postgres-source-api.md#constraint-source-spans).

TypeScript/JavaScript support the opt-in
[`query-reached-per-item`](rules/query-reached-per-item.md) rule through CLI and
asynchronous Node checks. It combines per-item AST positions with canonical
call paths and configured effect sinks, including transaction markers and batch
exemptions. Other languages do not yet expose equivalent call-policy facts.

Structured PostgreSQL source facts expose trigger event kinds and quoted UPDATE
OF identifiers, cast type components, literal EXECUTE concatenations, USING
expressions, unquoted INTO command terminators, and structural expression
completeness through the Rust/native and asynchronous Node CJS/ESM single/batch
APIs. Dynamic commands remain incomplete.

[Declared payload compatibility](rules/declared-payload-compatibility.md) is
language-neutral: explicit JSON Schema document pairs work for independently
compiled HTTP request/response or queue producers/consumers through Rust, CLI,
and asynchronous Node checks. Runtime endpoint adoption and inferred payload
values are outside this declaration-only boundary.

TS/JS coverage producers can opt into [assertion-owner checks](rules/coverage-producer-assertion-owner.md) through CLI and async Node `check()`, sharing prepared calls and explicit cross-process ownership.
Named imports through a sole plain external `export *` retain package/export
identity; local shadows, competing stars and unresolved cycles remain unknown.

TS/JS [forbidden lint suppression checks](rules/forbidden-lint-suppressions.md)
use AST comment facts through CLI and async Node `check()`/`analyzeProject()`.
Configured file globs, exact rule IDs, literal rule prefixes, and opt-in blanket
or TypeScript directive bans have the same finding locations and targets.

TS/JS production export consumers can be checked with explicit test/helper scope through [`production-exports-only-used-by-tests`](rules/production-exports-only-used-by-tests.md), available through CLI and async Node check reports. Static named aliases and re-exports preserve symbol ownership; dynamic or unresolved relationships follow the shared uncertainty policy. Exact `platformConsumedExports` entries can declare known platform consumers for selected names in root-relative files; sibling exports retain ordinary analysis. Next.js projects exempt the framework-consumed names on App Router, Pages Router, `proxy`, and `instrumentation-client` files. Identical star-reexport origins stay one symbol, and a tsconfig `paths` pattern match decides alias ownership.

Low-effort PostgreSQL query annotation also projects the fixed head of an exact trusted local SQL builder through direct `.text` executor reads, with paired `.values` reads, in CLI and asynchronous Node checks. Direct immutable string exports follow one prepared source import.

Low PostgreSQL annotation checks also project fixed heads from fresh source
helper records or trusted builder results through one local paired field read.
That pair may precede one executor options argument without changing the
already-read SQL text head.
A native builder's paired fields may also be read by a captured closure when
all uses of that builder preserve its fixed prefix; CLI and async Node checks
share the same prepared binding census.
Plain record text may come from an immutable local template, and an exact
`{ text, values }` extraction may feed one configured executor call.
Fresh trusted builder helper results may also flow through one safe local SQL
binding or a direct append at the executor; CLI and async checks share this
prepared source proof.
The CLI and async Node check use the same prepared facts.
They also share the low proof for an uninitialized local `let` assigned a fresh
native builder in each exhaustive adjacent arm and then read only by the next
configured executor. Both missing heads remain a finding; mixed or escaped
forms remain unresolved.

PostgreSQL annotation low effort shares the bounded awaited fresh native builder
return-origin proof across CLI, Rust checks and asynchronous Node checks. Exact
prepared imported append-effect obligations must all resolve; unsupported return,
await, alias or effect paths keep the existing unanalyzable policy.

TypeScript namespace bodies (nested, dotted, reopened, and `declare namespace`)
keep PostgreSQL executor and resource bindings local to each body. Namespace
`var` declarations hoist within that body without shadowing outer imports.


TS/JS [`csp-allowed-origins`](rules/csp-allowed-origins.md) checks third-party
browser origins (including protocol-relative `//host` and IPv6 literals) in JSX
elements and `fetch`/`WebSocket`/`EventSource` calls against static CSP
directives declared in `options.cspFiles` (first duplicate directive wins per
policy, multiple policies intersect, optional `options.selfOrigins`, shared
`options.unanalyzable` for dynamic/unparsed URLs, and module-level `const`
string arrays joined or spread in the same file or through one named import)
through Rust, CLI `check`, async Node `check()`, and batched `analyzeProject()`
checks.

TS/JS [`vitest-default-excludes`](rules/vitest-default-excludes.md) checks
literal discovery exclusions on a selected config (and its inline projects) and
on `test`-shaped object literals in config factory modules matched by
`options.files` through Rust, CLI `check`, async Node `check()`, and batched
`analyzeProject()` checks.

TS/JS [`vitest-project-policy`](rules/vitest-project-policy.md) checks per-project
literal properties through CLI `check`, asynchronous Node `check()`, and batched
`analyzeProject()` checks. Optional `requiredExcludeGlobs` applies the same
config selection to literal `test.exclude` arrays. Conditional and logical spreads of
literal objects resolve known keys, so an `exclude` key that no operand defines
stays omitted. It reads an explicit config and helper list, reuses
Vitest config parsing, and reports unresolved effective values as incomplete.
Helper globs select the prepared visible-file inventory. Optional
`declared.forbiddenValues` and `declared.forbidLiteralKinds` check all object
literals in config and helpers, including unimported helpers, without constant
propagation or incomplete reports for dynamic projects. Unreadable or malformed
selected sources produce incomplete evidence. Application include/exclude filters
and source suppressions apply to declaration diagnostics. Config is never executed
and the rule does not discover repository files.

Opt-in [`test-no-retries`](rules/test-no-retries.md) checks literal TypeScript
and JavaScript retry settings for Vitest, Playwright, and Jest, plus supported
GitHub Actions YAML steps. [`playwright-no-serial`](rules/playwright-no-serial.md)
checks Playwright serial settings in TypeScript and JavaScript. Rust, CLI
`check`, async Node `check()`, and batched `analyzeProject()` share both rules.

Opt-in [`postgres-lock-written-rows-only`](rules/postgres-lock-written-rows-only.md)
checks configured TypeScript/JavaScript PostgreSQL executors for unwritten-row,
table, constant advisory, and autocommit row locks. CLI `check` and async Node
`check()` share the same configuration, findings, advisories, and suppressions.
The rule recognizes resource-scoped transactions, hashed constant advisory keys,
static helper calls and re-exports, atomic same-statement lock/write CTEs, and
configured executor-option properties. Helper effects require directly awaited
calls with matching executor parameters. Cross-transaction order and
schema-dependent advisory checks remain outside scope.


The opt-in [`postgres-column-check-constraints`](rules/postgres-column-check-constraints.md)
rule shares catalog CHECK proofs across Rust, CLI `check`, async Node `check()`,
and prepared `analyzeProject()` check reports. Column patterns, integer types,
and the required predicate are explicit. Standard error severity applies;
`severity: advisory` is recommended for adoption. Unsupported CHECK expressions
leave the required proof missing.

The opt-in [grouping advice](rules/postgres-grouping-not-aggregation.md) shares
prepared catalog and statement facts between selected raw PostgreSQL and embedded
TS/JS SQL, including recovered template and `.append` fragments. All five shape
codes are always advisory and require a catalog; unsupported or unproven shapes
stay silent. CLI `check` and async Node `check` / `analyzeProject` share output.

The opt-in [`postgres-no-duplicate-keyed-lookups`](rules/postgres-no-duplicate-keyed-lookups.md)
rule shares prepared SQL statement facts across Rust, CLI `check`, async Node
`check()`, and `analyzeProject()` check reports for both embedded TS/JS SQL and
`sqlInclude` `.sql` files. It is off by default and uses the shared
`unanalyzable` policy and suppression directives.

The configured coverage assertion-owner analysis supports a narrow TS/JS scenario
table: unconditional no-argument member calls in a simple `const` `for...of` over a
closed local or module-private literal array of plain object rows with statically imported callbacks.
The CLI and async Node check APIs share this opt-in ownership projection. Ordinary
call-query output is unchanged; dynamic, mutated, escaped, filtered, or prematurely
exited tables remain unproven. See the [rule](rules/coverage-producer-assertion-owner.md).

The `test-no-live-repository-scan` rule shares owned temporary-root proof across
CLI and async Node checks: Node disposable directory `.path` values (including
`await using`) and same-file helpers with owned returns and no fallthrough.
See the [rule](rules/test-no-live-repository-scan.md) for the supported bounds.


Categorized local changes have CLI and async Node parity:
[`changes diff`](cli/changes-diff.md) maps to `changesDiff()`, and
[`tests plan --changes`](cli/tests-plan.md#semantic-change-reports) maps to
`testsPlan({ changes })`. The shared engine supports JavaScript package locks,
SwiftPM versions 1–3, NuGet framework keys, JSON, Apple strings and .NET resource
catalogs. Unsafe report input selects the full test suite.
