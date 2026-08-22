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
dedicated `no-mistakes python|go|rust|rails|php|java|kotlin|elixir` CLIs are later work. Agents
use `dependents --relationship <lang>` now and `tests plan python|go|cargo|rails|php|java|kotlin|elixir`
when those stacks are configured. Ecosystem lockfiles and dedicated language
CLIs are not started.

## Current Status

| Domain | Module graph | Test plan | HTTP routes | Queues | Status |
| --- | --- | --- | --- | --- | --- |
| TypeScript / JavaScript | yes | Vitest, Playwright, Jest | Express, Hono, Koa, Fastify, NestJS, Next.js, Remix file routes | BullMQ, glide-mq | shipped (tRPC procedures opt-in) |
| Swift | `swift-import`, `swift-ref`, `swift-package` | `tests plan swift` | no (client `http` edges only) | no | shipped, narrower |
| .NET / C# | `dotnet-using`, `dotnet-ref`, `dotnet-project` | `tests plan dotnet` | ASP.NET `MapGet` / `[HttpGet]` literals | no | shipped (v1 extractors + plan) |
| Python, Django, Celery | `python-import`, `python-ref` | `tests plan python` | Django `path(`, Flask, FastAPI | Celery `.delay(` / `@shared_task` | shipped (v1 extractors + plan) |
| Go, Asynq | `go-import`, `go-ref` | `tests plan go` | net/http, Chi, Gin, Echo, Fiber literals | Asynq `NewTask` / `HandleFunc` | shipped (v1 extractors + plan) |
| Kafka | n/a | n/a | n/a | static topic produce/consume | shipped (v1 extractors) |
| Rust | `rust-use`, `rust-mod` | `tests plan cargo` | Axum, Actix, Rocket literals | no | shipped (v1 extractors + plan) |
| Ruby on Rails | `ruby-require`, `ruby-ref` | `tests plan rails` | `routes.rb` `to:` / `resources` | Active Job `perform_later`, Sidekiq `perform_async` | shipped (v1 extractors + plan) |
| PHP | `php-use`, `php-package` | `tests plan php` | Laravel `Route::` / `Route::resource` or Symfony attribute/YAML | Laravel `::dispatch` / `ShouldQueue` or Symfony Messenger | shipped (v1 extractors + plan) |
| Java, Spring | `java-import`, `java-ref` | `tests plan java` | Spring `@RequestMapping` / `@GetMapping` literals | no | shipped (v1 extractors + plan) |
| Kotlin, Spring | `kotlin-import`, `kotlin-ref` | `tests plan kotlin` | Spring `@RequestMapping` / `@GetMapping` literals on `.kt` | no | shipped (v1 extractors + plan) |
| Elixir, Phoenix | `elixir-import`, `elixir-ref` | `tests plan elixir` | Phoenix `get`/`post`/`put`/`patch`/`delete` literals | no | shipped (v1 extractors + plan) |

CI workflows and Terraform/OpenTofu are adjacent graph domains, not language
frontends. They stay available to every language once files are tracked.

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
There is still no `no-mistakes rust` CLI.

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
TS/JS producers to virtual job nodes to workers. Celery, Asynq, Kafka, Active
Job, Sidekiq, Laravel, and Symfony Messenger emit the same `queue-enqueue` /
`queue-worker` edges into `DepGraph`; query those with
`dependents --relationship queue`. The dedicated `queues` commands project
those language edges into the existing report. They do not get private graph
shapes.

**HTTP clients.** Static client calls produce `http` edges to matching route
files, the same way TS `fetch` and Swift `Endpoint` literals do.

**Lockfiles.** `lockfile diff` today parses npm-family lockfiles. Language
support adds the ecosystem lockfile when agents need package-change impact:
`poetry.lock` / `uv.lock` / `Pipfile.lock`, `go.mod`, `Cargo.lock`,
`Gemfile.lock`, `composer.lock`. Go package-change impact reads the selected
module graph from `go.mod`, not authentication hashes in `go.sum`.

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

Counterexample: a `no-mistakes python` command that walks the tree, parses
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

```csharp
app.MapGet("/users", UserHandlers.ListUsers);
[HttpGet("/orders")]
public object GetOrders() => new object();
```

Configure projects through `tests.dotnet.projects` the same way `tests plan
dotnet` already does. Empty project lists disable the extractor; there is no
hardcoded `Controllers/` glob.

## Python, Django, Flask, FastAPI, Celery

Python support is the language frontend. Django, Flask, FastAPI, and Celery
are configured domain extractors on top of it.

| Feature | TS/JS reference | Python equivalent |
| --- | --- | --- |
| Module graph | `import` / `require` | `import`, `from … import`, relative `.` / `..` imports |
| Package identity | `package.json` workspaces | configured package roots; `pyproject.toml` / `setup.cfg` names |
| Symbols | exports and importers | module-level `def` / `class` and qualified references |
| Tests | `tests plan vitest` | `tests plan python` over pytest / unittest files |
| HTTP routes | Express / Hono / Koa | Django URLconf → view, plus configured Flask / FastAPI decorator literals |
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
@app.route(prefix + "/users")
@router.post(prefix + "/items")
```

Django settings, middleware, and model graphs are out of scope for the first
cut unless they are needed to resolve a static import or route.

## Go, Asynq, HTTP

Go support is the language frontend. Asynq is the queue extractor. Configured
`net/http`, Chi, Gin, Echo, and Fiber string-literal registrations emit
`RouteRef` edges.

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
http.HandleFunc("/health", Health)
r.Get("/users", Users)
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

| Feature | TS/JS reference | Rails equivalent |
| --- | --- | --- |
| Module graph | `import` | `require`, `require_relative`, Zeitwerk-constant references inside configured app roots |
| Package identity | workspace packages | configured engine/app roots and `Gemfile` path gems |
| Tests | `tests plan vitest` | `tests plan rails` over Minitest / RSpec files |
| HTTP routes | `server routes` | configured `config/routes.rb` (and engine routes) → controller#action, including bare `resources :name` |
| Queues | BullMQ | Active Job `SomeJob.perform_later` or Sidekiq `SomeWorker.perform_async` → job class |
| Lockfile | npm-family | `Gemfile.lock` |

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

| Feature | TS/JS reference | PHP equivalent |
| --- | --- | --- |
| Module graph | `import` | `use`, `require`/`include` of local files, PSR-4 from configured `composer.json` |
| Package identity | workspace packages | configured Composer packages / path repositories |
| Tests | `tests plan vitest` | `tests plan php` over PHPUnit / Pest files |
| HTTP routes | `server routes` | configured Laravel `Route::` / bare `Route::resource` or Symfony attribute/YAML routes |
| Queues | BullMQ | Laravel `SomeJob::dispatch()` / `ShouldQueue`, or Symfony Messenger handlers |
| Lockfile | npm-family | `composer.lock` |

```php
Route::get('/api/users', [UserController::class, 'index']);
Route::resource('users', UserController::class);
SomeJob::dispatch($user);
#[Route('/health', methods: ['GET'])]
class HealthController {}
$bus->dispatch(new WelcomeMessage());
```

`__DIR__ . '/' . $name`, `app($abstract)`, `#[Route($prefix . '/users')]`,
`$bus->dispatch($message)` lookups, `Route::resource(..., ['only' => ...])`,
`Route::resource(...)->only([...])`, nested dotted `Route::resource` names,
named-argument `Route::resource` calls, and `Route::apiResource` are non-edges. Missing `tests.php.framework`
still enables neither Laravel nor Symfony extractors.

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

| Feature | TS/JS reference | Java equivalent |
| --- | --- | --- |
| Module graph | `import` | exact `import com.example.User;` |
| Package identity | workspace packages | configured `tests.java.packages` |
| Tests | `tests plan vitest` | `tests plan java` over `*Test.java` / `*Tests.java` / `*IT.java`; `mvn test [-f <package>/pom.xml] -Dtest=` |
| HTTP routes | `server routes` | Spring `@RequestMapping` + `@GetMapping` literals |
| Queues | BullMQ | no |
| Lockfile | npm-family | later (`pom.xml` native fallback only) |

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
explicit `import`, and `import com.example.*` are non-edges.

## Kotlin, Spring

Kotlin support is a language frontend for configured `tests.kotlin.packages`.
Empty lists disable the extractor; there is no Gradle inference and the list
is not folded into `tests.java`. Exact `import com.example.User` statements
emit `kotlin-import`. Star imports are non-edges. Optional `as` aliases still
record the original fully-qualified class name. Class/interface/object names plus capitalized
identifiers emit `kotlin-ref`.

Spring HTTP v1 reuses the Java mapping literals on `.kt` files, matching
`fun listUsers()` handlers after `@GetMapping("/users")`.

| Feature | TS/JS reference | Kotlin equivalent |
| --- | --- | --- |
| Module graph | `import` | exact `import com.example.User` |
| Package identity | workspace packages | configured `tests.kotlin.packages` |
| Tests | `tests plan vitest` | `tests plan kotlin` over `*Test.kt` / `*Tests.kt` / `*IT.kt`; `gradle [-p <package>] test --tests` |
| HTTP routes | `server routes` | Spring `@RequestMapping` + `@GetMapping` literals on `.kt` |
| Queues | BullMQ | no |
| Lockfile | npm-family | later (`build.gradle.kts` native fallback only) |

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
non-edges. Native fallback is `build.gradle` / `build.gradle.kts` plus
non-test `.kt` files; `settings.gradle*` is not a trigger. `--tests` uses the
file stem, matching Java `-Dtest`.

## Elixir, Phoenix

Elixir support is a language frontend for configured `tests.elixir.apps`.
Empty lists disable the extractor; there is no `mix.exs` inference.
Exact `alias`/`import`/`use MyApp.User` statements emit `elixir-import`.
Brace aliases `alias MyApp.{User, Role}` and wildcard imports are non-edges.
Module names plus capitalized identifiers emit `elixir-ref`. Same-package
refs without an `alias`/`import`/`use` are non-edges.

Phoenix HTTP v1 matches literal `get "/users", Controller, :index` (and
`post`/`put`/`patch`/`delete`) registrations. `resources` macros and
`scope "/api"` prefix joining are non-edges.

| Feature | TS/JS reference | Elixir equivalent |
| --- | --- | --- |
| Module graph | `import` | exact `alias`/`import`/`use MyApp.User` |
| Package identity | workspace packages | configured `tests.elixir.apps` |
| Tests | `tests plan vitest` | `tests plan elixir` over `*_test.exs`; `mix test <path>` (umbrella child paths stay repo-relative) |
| HTTP routes | `server routes` | Phoenix `get`/`post`/`put`/`patch`/`delete` literals |
| Queues | BullMQ | no |
| Lockfile | npm-family | later (`mix.exs` native fallback only) |

```elixir
defmodule MyAppWeb.Router do
  get "/users", MyAppWeb.UserController, :index
  post "/users", MyAppWeb.UserController, :create
end
```

Brace aliases, same-package refs without `alias`/`import`/`use`, Phoenix
`resources` macros, `scope "/api"` prefix joining, `mix.lock` / `config.exs`
native fallback, queues, and a dedicated `no-mistakes elixir` CLI are
non-edges / later work. Native fallback is `mix.exs` plus non-test `.ex`
files under configured apps. Any `.ex`/`.exs` under `/test/` is
non-production. `*_test.exs` is the test suffix.

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
- `java` — Java import and reference edges
- `kotlin` — Kotlin import and reference edges
- `elixir` — Elixir import and reference edges

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
    trpc:
      routers: ["src/trpc/**/*.ts"]
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
```

Counterexample: defaulting to “every `urls.py`, every `go.mod`, every Rails
`app/`” when the lists are omitted. Full-suite and repository-wide scans stay
opt-in.

## Agent Fallback

v1 module graphs, `tests plan <lang>`, and named route/queue extractors are
shipped for configured Python, Go, Rust, Rails, PHP, Java, Kotlin, and Elixir packages. Use
`dependents --relationship <lang|route|queue>` and
`tests plan python|go|cargo|rails|php|java|kotlin|elixir` for those questions instead of `rg`.

Keep using `rg` for holes the status table still marks `no` or later:
ecosystem lockfile diffs (`poetry.lock`, `uv.lock`, `Pipfile.lock`, `go.mod`,
`Cargo.lock`, `Gemfile.lock`, `composer.lock`), language HTTP clients, Laravel
`Route::resource` `only`/`except`, nested dotted names, named arguments, and `Route::apiResource`, Kafka
outside TS/Python literal shapes, language `symbols`/`call-sites`, and
dedicated `no-mistakes python|go|rust|rails|php|java|kotlin|elixir` CLIs.

See [Architecture](architecture.md) for the one-pass session rules,
[Graph edges](graph-edges.md) for the current edge kinds, and
[Tests and selectors](configuration/tests.md) for how Swift and .NET
already require explicit package/project lists.
