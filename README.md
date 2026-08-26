# no-mistakes

[![CodSpeed](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://app.codspeed.io/jonathanong/no-mistakes?utm_source=badge)

`no-mistakes` is deterministic codebase intelligence for coding agents. It
answers structural questions, selects affected tests, and enforces AST-friendly
repository rules without running the application, calling an AI model, or
maintaining a database.

The canonical graph covers TypeScript/JavaScript, React, Next.js, Playwright,
queues, server routes, GitHub Actions, Terraform/OpenTofu, Swift, .NET, and
explicitly configured Python, Go, Rust, Ruby, PHP, Java, Kotlin, Elixir, and
Dart projects. Results are small, structured, and designed to feed directly
into the next edit or validation step.

The primary use cases are:

1. Discovering impacted files and tests during planning
2. Running selected tests in PR CI to minimize CI costs
3. AST-based guardrails that keep code analyzable as agents modify it

The practical payoff is a fast, repeatable change map: agents can discover the
right tests and callers locally, CI can spend work only where a change reaches,
and teams can inspect the same typed graph from the CLI or async Node API.
Configuration keeps project-specific conventions explicit instead of hiding
them in a remote service or a persistent index.

For example, consider this dependency chain:

> Backend `getPost(id)` -> Backend GET `/posts/:id` -> Next.js Fetch GET `/posts/:id` -> Next.js Page `/post/[id]` -> Playwright Test on `/post/[id]`

One graph query can expose the whole chain while planning. In CI, a change to
`getPost()` can select the relevant Playwright tests instead of running every
suite. The answer is derived locally from syntax and configuration—no
embeddings or probabilistic inference.

The same analysis powers repository-wide rules that ESLint and Oxlint cannot
express one file at a time. One example is detecting duplicate exported
function names that an agent may otherwise recreate after an incomplete text
search:

```ts
// backend/controllers/users.mts
export function getCurrentUser (ctx) {
  return UserService.getUserById(ctx.params.id)
}

// backend/controllers/getCurrentUser.mts
export function getCurrentUser (ctx) {
  return UserService.getUserById(ctx.params.id)
}
```

`no-mistakes check` reports both definitions so the agent can reuse or rename
the existing API.

## Why AST-based?

Many codebase-intelligence tools build persistent indexes, vector embeddings,
or an LLM layer. Those approaches add cost, operational state, and awkwardness
when several worktrees are active. `no-mistakes` parses the current checkout on
demand in Rust, builds one in-memory graph, and discards it after the request.

There are a few trade-offs with this approach:

1. Dynamic imports, selectors, routes, and queue names cannot always be resolved.
   The bundled rules help keep important relationships static and analyzable.
2. Some relationships are heuristic. The engine favors recall, so agents should
   confirm exact call sites with `rg` after the graph narrows the file set.
3. On-demand parsing is CPU-intensive. A machine-wide lock prevents competing
   worktrees from running large analyses concurrently, and the async Node API
   can batch related reports into one prepared request.
4. Small modules and direct, static bindings produce the most useful graph.

## Agent Workflows

| Agent question | Use |
| --- | --- |
| What does this file import? | `no-mistakes dependencies <file> --format json` |
| What can this change affect? | `no-mistakes dependents <file> --format paths` |
| Who uses this export? | `no-mistakes dependents <file>#Symbol --format json` |
| What does a signature change affect? | `no-mistakes symbols <file> --mode signature-impact --symbol Symbol --format json` |
| What does this module export/import? | `no-mistakes symbols <file> --include both --format json` |
| Which packages do source files import directly? | `no-mistakes import-usages --root . --filter 'src/**' --format json` |
| Which tests should run? | `no-mistakes tests plan <playwright\|vitest> --changed-file <file> --format json` |
| Why was a test selected? | `no-mistakes tests why <test> --plan plan.json` |
| Which Playwright tests cover this page? | `no-mistakes playwright related <file> --json` |
| Which queue/server files are connected? | `no-mistakes queues related <file> --json`; `no-mistakes server related <file> --json` |
| Are configured repository rules passing? | `no-mistakes check --format json` |

Use `--format json` when an agent will parse the answer, `--format paths` when
the output feeds another shell command, and `--timings` when explaining analysis
cost. For repeated in-process queries, prefer the async Node API so one agent
workflow can avoid subprocess overhead.

### Example recipes

| Goal | Command |
|---|---|
| Check if a named export is still used (static imports) | `no-mistakes dead-exports <file> [NAME...]` |
| Find all Vitest tests covering a component | `no-mistakes tests plan vitest --changed-file <file> --format paths` |
| Select tests from a git range | `no-mistakes tests plan vitest --from-git-diff origin/main...HEAD --format json` |
| Find all Playwright tests covering a route/page | `no-mistakes tests plan playwright --changed-file <file> --format paths` |
| Find direct importers before renaming a module | `no-mistakes dependents <file> --depth 1 --relationship import --relationship workspace --format paths` |
| Count static-import callers of a file | `no-mistakes importers <file>` |

## Install

```sh
npm install --save-dev no-mistakes eslint-plugin-no-mistakes
```

Local development from this repository:

```sh
cargo run -p no-mistakes -- dependents src/utils.mts --format paths
```

## Documentation

- [Documentation index](docs/README.md)
- [CLI commands](docs/cli/README.md)
- [Node/N-API guide](docs/node-api.md)
- [Configuration](docs/configuration/README.md)
- [Graph edge types](docs/graph-edges.md)
- [Feature parity](docs/feature-parity.md)
- [no-mistakes rules](docs/rules/README.md)
- [ESLint rules](docs/eslint-rules/README.md)
- [Agent guide](docs/agent-guide.md)
- [Packaged agent skill](skills/no-mistakes/SKILL.md)
- [AST analysis behavior](docs/ast-analysis.md)

## Contributing

This repository is a huge token sink. Thus, contributions are welcomed.

1. Please add test cases in `test-cases/`
2. Annotate which AI harness + model was used, `Co-Authored-By` is preferred
3. Maintain 99% project and patch test coverage

## Support

| Language/Framework/Tool | Status |
| -- | -- |
| TypeScript | Mature |
| Next.js / React / Playwright | Mature |
| `pnpm`, `npm`, `yarn`, `bun` | Supported, primarily tested using `pnpm` |
| `bullmq`, `glide-mq` | Supported, primarily tested for `glide-mq` |
| Vitest / Jest | Supported |
| Queue/server/CI/Terraform/OpenTofu | Supported with explicit configuration |
| Swift / .NET | Shipped, narrower language frontends |
| Python / Go / Rust / Ruby / PHP / Java / Kotlin / Elixir / Dart | Shipped v1 frontends and test plans; configure package or module roots explicitly |

## Design Constraints

- Local and deterministic: no services, databases, remote AI calls, or
  persistent filesystem caches.
- One pass per invocation: discover files once, read each requested source
  once, parse it once per required semantic mode, and reuse shared fact maps
  across checks.
- Programmatic parity: stable CLI capabilities also expose async N-API
  functions for Node callers.
- Explicit configuration: route roots, queue factories, test projects, and
  global fallback behavior are opt-in configuration, not inferred conventions.

## Harness Ecosystem

This is part of the following harness ecosystem:

- [auto-harness](https://github.com/jonathanong/auto-harness) - non-interactive agent CLI orchestration across sandboxes
- [agent-blackboard](https://github.com/jonathanong/agent-blackboard) - session-scoped telemetry for autonomous agents
- [pr-shepherd](https://github.com/jonathanong/pr-shepherd) - autonomous pull request shepherd
- [no-mistakes](https://github.com/jonathanong/no-mistakes) - deterministic AST-based codebase intelligence, test selection, and linting for agents
