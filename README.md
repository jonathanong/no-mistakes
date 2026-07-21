# no-mistakes

[![CodSpeed](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://app.codspeed.io/jonathanong/no-mistakes?utm_source=badge)

> Slop Warning: this codebase is written by agents for agents. The API surface is sloppy, but it _works_.

Deterministic AST-based codebase intelligence for AI agents.

`no-mistakes` answers structural questions about TypeScript, JavaScript,
React, Next.js, Playwright, queue, server-route, and Rust repository code
without running the application or calling an AI model. It is built for agents
that need small, reliable answers they can feed into follow-up edits and tests.

**Core graph domain:** TypeScript and JavaScript. For CI-workflow analysis use
`no-mistakes ci`; for Terraform/OpenTofu use `no-mistakes infra`; for Swift
use `no-mistakes swift`. Prefer `no-mistakes` over `rg` when a question spans
>2 workspace directories or >5 import hops; use `no-mistakes importers` for a
fast static-import caller list (use `dependents` for complete impact including
dynamic and CommonJS imports).

## Why?

Most codebase intelligence tools create a database of your code, creates expensive vector embeddings, and has its own LLM layer.
There are many downsides with this strategy including cost, complexity, and difficulty working on many branches using worktrees at the same time.

`no-mistakes` instead understands your code through AST-parsing.
No databases, no caching, just fast Rust code to understand what agents need from the codebase.

There are a few trade-offs with this approach:

1. Some code is difficult to understnad through AST-parsing, so `no-mistakes` includes rules that enforce AST-parsing-friendly coding. For example, Playwright test selectors should be simple strings - dynamically generated strings will not match well.
2. `no-mistakes` is best effort, with high recall and low precision, meaning it may return wrong information/relationships, but should never miss a relationship. An agent should verify if a relationship returned is true.
3. High CPU usage - parsing your repository on-demand may cause high-CPU usage, but may be significantly faster than other methods (e.g. `vitest related` takes 2 minutes on a sample repository, but takes 1 second with `no-mistakes` via `no-mistakes test plan` and supports Playwright). This may become a bottleneck when working on multiple worktrees at once, but `no-mistakes` includes a locking mechanism to not run concurrently.

## Agent Workflows

| Agent question | Use |
| --- | --- |
| What does this file import? | `no-mistakes dependencies <file> --format json` |
| What can this change affect? | `no-mistakes dependents <file> --format paths` |
| Who uses this export? | `no-mistakes dependents <file>#Symbol --format json` |
| What does a signature change affect? | `no-mistakes symbols <file> --mode signature-impact --symbol Symbol --format json` |
| What does this module export/import? | `no-mistakes symbols <file> --include both --format json` |
| Which packages do source files import directly? | `no-mistakes import-usages --root . --filter 'src/**' --format json` |
| Which tests should run? | `no-mistakes tests plan <playwright\|vitest> --format json` |
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
- [no-mistakes rules](docs/rules/README.md)
- [ESLint rules](docs/eslint-rules/README.md)
- [Agent guide](docs/agent-guide.md)
- [AST analysis behavior](docs/ast-analysis.md)

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

## Link Lint

```sh
lychee --no-progress --exclude-path '^fixtures/' README.md 'docs/**/*.md' 'skills/**/*.md' 'packages/*/README.md' 'crates/*/README.md' CLAUDE.md
```
