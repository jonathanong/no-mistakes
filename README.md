# no-mistakes

Deterministic AST-based codebase intelligence for AI agents.

`no-mistakes` answers structural questions about TypeScript, JavaScript,
React, Next.js, Playwright, queue, server-route, and Rust repository code
without running the application or calling an AI model. It is built for agents
that need small, reliable answers they can feed into follow-up edits and tests.

**Core graph domain:** TypeScript and JavaScript. For CI-workflow analysis use
`no-mistakes ci`; for Terraform/OpenTofu use `no-mistakes infra`; for Swift
use `no-mistakes swift`. Prefer `no-mistakes` over `rg` when a question spans
>2 workspace directories or >5 import hops; use `no-mistakes importers` for a
fast single-file direct-caller list and count.

## Agent Workflows

| Agent question | Use |
| --- | --- |
| What does this file import? | `no-mistakes dependencies <file> --format json` |
| What can this change affect? | `no-mistakes dependents <file> --format paths` |
| Who uses this export? | `no-mistakes dependents <file>#Symbol --format json` |
| What does a signature change affect? | `no-mistakes symbols <file> --mode signature-impact --symbol Symbol --format json` |
| What does this module export/import? | `no-mistakes symbols <file> --include both --format json` |
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
| Check if an export is dead across the monorepo | `no-mistakes dead-exports <file> [NAME...]` |
| Find all test files that cover a component | `no-mistakes tests plan vitest --changed-file <file> --format paths` |
| Find all files importing a module (e.g., before renaming) | `no-mistakes dependents <file> --format paths` |
| Count direct importers of a file | `no-mistakes importers <file>` |

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
- One pass per invocation: discover files once, parse TS/JS once for requested
  facts, and reuse shared fact maps across checks.
- Programmatic parity: stable CLI capabilities also expose async N-API
  functions for Node callers.
- Explicit configuration: route roots, queue factories, test projects, and
  global fallback behavior are opt-in configuration, not inferred conventions.

## Link Lint

```sh
lychee --no-progress --exclude-path '^fixtures/' README.md 'docs/**/*.md' 'skills/**/*.md' 'packages/*/README.md' 'crates/*/README.md' CLAUDE.md
```
