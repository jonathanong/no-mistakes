# Feature Parity

`no-mistakes` is a local, deterministic codebase-intelligence engine. TypeScript
and JavaScript are the reference implementation for the full product surface.
This page is the contract for bringing other languages and their key frameworks
up to that surface.

A language or framework is supported when an agent can ask the same structural
questions it already asks of TS/JS, get deterministic structured output, and
do so without shelling out to `rg` for the graph itself.

v1 is the Swift/.NET bar plus the named key feature for each stack: a module
graph, `tests plan`, and either HTTP routes or queues. Playwright, React,
Next.js fetches, call-sites, dead-exports, ecosystem lockfile diffs, and
dedicated `no-mistakes python|go|rust|rails|php` CLIs are later work. Agents
use `dependents --relationship <lang>` and `tests plan <lang>` once those
edges and planners ship.

## Current Status

| Domain | Module graph | Test plan | HTTP routes | Queues | Status |
| --- | --- | --- | --- | --- | --- |
| TypeScript / JavaScript | yes | Vitest, Playwright | Express, Hono, Koa, Next.js | BullMQ, glide-mq | shipped |
| Swift | `swift-import`, `swift-ref`, `swift-package` | `tests plan swift` | no (client `http` edges only) | no | shipped, narrower |
| .NET / C# | `dotnet-using`, `dotnet-ref`, `dotnet-project` | `tests plan dotnet` | no | no | shipped, narrower |
| Rust | no | `--test cargo` globs only | no | no | partial: project type, check rules, CI Cargo edges |
| Python, Django, Celery | no | no | no | no | not started |
| Go, Asynq | no | no | no | no | not started |
| Kafka | n/a | n/a | n/a | no | not started |
| Ruby on Rails | no | no | no | no | not started |
| PHP | no | no | no | no | not started |

CI workflows and Terraform/OpenTofu are adjacent graph domains, not language
frontends. They stay available to every language once files are tracked.

Rust today is not a language frontend. `projects.*.type: rust` exists,
[`rust-max-lines-per-file`](rules/rust-max-lines-per-file.md),
[`rust-no-inline-allows`](rules/rust-no-inline-allows.md), and
[`rust-no-inline-tests`](rules/rust-no-inline-tests.md) run as filesystem
checks, `--test cargo` filters `**/tests/**/*.rs` and `src/**/*_test.rs`, and
`ci` edges connect GitHub Actions workflows to Rust binaries invoked by
supported Cargo commands. There is no `use`/`mod` graph, no `tests plan cargo`,
and no Rust CLI.

## Canonical Feature Set

These are the features a new language must cover, or explicitly decline with a
documented limit. The TS/JS command is the reference behavior.

**Module graph.** `dependencies`, `dependents`, and `related` over typed
edges in the canonical `DepGraph`. TS/JS uses `import`, `type-import`,
`dynamic-import`, `require`, `workspace`, and `package`. Swift and .NET
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
`server contracts` list configured route definitions and static client calls.
Do not invent a second route graph.

**Queues.** `queues edges`, `queues related`, and `queues check` connect
producers to virtual job nodes to workers. Celery, Asynq, and Kafka extend this
domain. They do not get private graph shapes.

**HTTP clients.** Static client calls produce `http` edges to matching route
files, the same way TS `fetch` and Swift `Endpoint` literals do.

**Lockfiles.** `lockfile diff` today parses npm-family lockfiles. Language
support adds the ecosystem lockfile when agents need package-change impact:
`poetry.lock` / `uv.lock` / `Pipfile.lock`, `go.mod`, `Cargo.lock`,
`Gemfile.lock`, `composer.lock`. Go package-change impact reads the selected
module graph from `go.mod`, not authentication hashes in `go.sum`.

**Checks, CLI, and N-API.** Every stable CLI capability needs an async N-API
equivalent, fixture-backed tests, and docs. Language-specific check rules
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
   `bin/rails test` / `rspec`, `phpunit` / `artisan test`).
6. Ship CLI, N-API, `docs/cli/*`, `docs/graph-edges.md`,
   `docs/configuration/tests.md`, and fixtures in the same change.

Counterexample: a `no-mistakes python` command that walks the tree, parses
files again, and builds a standalone import graph. That violates prepared
analysis ownership.

## Python, Django, Celery

Python support is the language frontend. Django and Celery are configured
domain extractors on top of it.

| Feature | TS/JS reference | Python equivalent |
| --- | --- | --- |
| Module graph | `import` / `require` | `import`, `from … import`, relative `.` / `..` imports |
| Package identity | `package.json` workspaces | configured package roots; `pyproject.toml` / `setup.cfg` names |
| Symbols | exports and importers | module-level `def` / `class` and qualified references |
| Tests | `tests plan vitest` | `tests plan python` over pytest / unittest files |
| HTTP routes | Express / Hono / Koa | Django URLconf → view, plus Flask / FastAPI if configured |
| Queues | BullMQ / glide-mq | Celery `@shared_task` / `@app.task`, `.delay(` / `.apply_async(` |
| Lockfile | pnpm / npm / yarn / bun | `poetry.lock`, `uv.lock`, `Pipfile.lock` |

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
```

Django settings, middleware, and model graphs are out of scope for the first
cut unless they are needed to resolve a static import or route.

## Go, Asynq

Go support is the language frontend. Asynq is the queue extractor.

| Feature | TS/JS reference | Go equivalent |
| --- | --- | --- |
| Module graph | file imports | `import` of local packages from configured `go.mod` modules |
| Package identity | workspace packages | `go.mod` module path plus configured package directories |
| Symbols | named exports | exported (`Uppercase`) funcs/types and references |
| Tests | `tests plan vitest` | `tests plan go` → `go test` in owning packages |
| HTTP routes | `server routes` | configured `net/http`, Chi, Gin, Echo, or Fiber registrations |
| Queues | BullMQ job name | Asynq `NewTask("mail:welcome", …)` / `HandleFunc("mail:welcome", …)` |
| Lockfile | npm-family | `go.mod` (selected module graph, not `go.sum`) |

Asynq task type strings are the virtual job identity, same as a BullMQ job
name. A producer file gets `queue-enqueue`; the handler file gets
`queue-worker`.

```go
client.Enqueue(asynq.NewTask("mail:welcome", payload), asynq.Queue("default"))
mux.HandleFunc("mail:welcome", HandleWelcome)
```

Computed task types and `http.Handle(pattern, …)` where `pattern` is not a
string literal produce no edge. `go/ast` in-process is acceptable; invoking the
`go` tool for analysis is not.

## Kafka

Kafka is a queue backend, not a language. Producers and consumers in any
supported language emit the existing `queue-enqueue` and `queue-worker` edges.
The virtual node identity is `<cluster>:<topic>`: an explicit configured
logical cluster or broker namespace plus the static topic string. Topic names
are cluster-scoped, so two clients that reuse `mail.welcome` on different
clusters stay distinct. A consumer group id is edge metadata (or a downstream
node), never part of that identity.

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

Rust should become a language frontend at Swift/.NET depth, then pick up
routes and queues from the shared domains.

| Feature | TS/JS reference | Rust equivalent |
| --- | --- | --- |
| Module graph | `import` | `mod`, `use crate::…`, `use super::…`, path attrs |
| Package identity | workspace packages | configured `Cargo.toml` packages and path deps |
| Symbols | named exports | `pub fn` / `pub struct` / `pub enum` and `use` paths |
| Tests | `tests plan vitest` | `tests plan cargo` → `cargo test -p <pkg>` for sibling `tests.rs`; `cargo test -p <pkg> --test <name>` only for `tests/` integration targets |
| HTTP routes | `server routes` | configured Axum, Actix, or Rocket registrations |
| Queues | BullMQ | configured enqueue/worker globs; Kafka when present |
| Lockfile | npm-family | `Cargo.lock` |
| Checks | `unique-exports` | keep the existing `rust-*` filesystem rules |

`ci` remains the narrow workflow-file → Rust-binary Cargo edge. Do not overload
it with `use`/`mod` edges. `workflow` already resolves supported Cargo
`run:` lines.

Inline `#[cfg(test)]` modules are banned by `rust-no-inline-tests` in this
repo; discovery should prefer `tests/**/*.rs` and sibling `tests.rs` files.

## Ruby on Rails

Rails support is Ruby module facts plus configured route and Active Job
extractors.

| Feature | TS/JS reference | Rails equivalent |
| --- | --- | --- |
| Module graph | `import` | `require`, `require_relative`, Zeitwerk-constant references inside configured app roots |
| Package identity | workspace packages | configured engine/app roots and `Gemfile` path gems |
| Tests | `tests plan vitest` | `tests plan rails` over Minitest / RSpec files |
| HTTP routes | `server routes` | configured `config/routes.rb` (and engine routes) → controller#action |
| Queues | BullMQ | Active Job `SomeJob.perform_later` → job class. Sidekiq `perform_async` is a later extractor, not v1. |
| Lockfile | npm-family | `Gemfile.lock` |

Zeitwerk inference is heuristic and must stay inside configured roots. Do not
scan the whole repository for `app/models`. Dynamic `constantize`,
`send(:"#{name}_path")`, and `perform_later` on a computed job class produce
no edge.

```ruby
get "/api/users", to: "users#index"
WelcomeJob.perform_later(user)
```

## PHP

PHP support is Composer/PSR-4 facts plus one configured framework extractor.
Set `tests.php.framework` to `laravel` or `symfony`. Do not infer the
framework from files, and do not enable both extractors from a missing value.

| Feature | TS/JS reference | PHP equivalent |
| --- | --- | --- |
| Module graph | `import` | `use`, `require`/`include` of local files, PSR-4 from configured `composer.json` |
| Package identity | workspace packages | configured Composer packages / path repositories |
| Tests | `tests plan vitest` | `tests plan php` over PHPUnit / Pest files |
| HTTP routes | `server routes` | configured Laravel `Route::` or Symfony attribute/YAML routes |
| Queues | BullMQ | Laravel `SomeJob::dispatch()` / `ShouldQueue`, or Symfony Messenger handlers |
| Lockfile | npm-family | `composer.lock` |

```php
Route::get('/api/users', [UserController::class, 'index']);
SomeJob::dispatch($user);
```

`__DIR__ . '/' . $name` and `app($abstract)` lookups are non-edges.

## Shared Domain Rules

Route, queue, and Kafka extractors are language-specific visitors that emit
the same public edge kinds (`server-route`, `client-call`, `http`,
`queue-enqueue`, `queue-worker`). A Celery producer in Python and an Asynq
producer in Go must be traversable with `--relationship queue`.

Relationship filters for the language graph itself follow Swift/.NET:

- `python` — Python import and reference edges
- `go` — Go import and reference edges
- `rust` — Rust `use`/`mod` and path-dep edges
- `ruby` — Ruby require and configured constant edges
- `php` — PHP `use` / Composer edges

Additive language flags must not change existing TS/JS report fields. When a
broader resolver catalog is needed (for example tests that live outside the
app package), add a fixture-backed parity test that compares baseline fields
with the flag off and on.

## Configuration Shape

New knobs are explicit and scoped. The names below are the intended contract;
they do not exist until the corresponding implementation ships.

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
```

Counterexample: defaulting to “every `urls.py`, every `go.mod`, every Rails
`app/`” when the lists are omitted. Full-suite and repository-wide scans stay
opt-in.

## Agent Fallback

Until a row in the status table is `shipped`, agents should keep using `rg`
for that language. The shipped `no-mistakes` skill already says Go and Rust
sources have no import-graph domain. That remains correct until the graph
edges and test planner land.

See [Architecture](architecture.md) for the one-pass session rules,
[Graph edges](graph-edges.md) for the current edge kinds, and
[Tests and selectors](configuration/tests.md) for how Swift and .NET
already require explicit package/project lists.
