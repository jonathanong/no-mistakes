# no-mistakes

Deterministic AST-based codebase intelligence for humans and AI agents.

This repository contains a Rust CLI, npm packages, ESLint/Oxlint plugins, and
Codex skills for answering structural questions about TypeScript, JavaScript,
React, Next.js, Playwright, queue, and server-route code without running the
application or calling an AI model.

## Start Here

The canonical documentation lives in [docs/](docs/README.md):

- [Documentation index](docs/README.md)
- [CLI reference](docs/cli-reference.md)
- [AST analysis behavior](docs/ast-analysis.md)
- [Agent guide](docs/agent-guide.md)
- [ESLint and Oxlint plugins](docs/eslint-plugin.md)

## Tools

| Tool | Purpose |
| --- | --- |
| `no-mistakes` | Unified codebase graph, symbols, Playwright coverage, fetch, React, queue, server-route, and check commands. |
| `eslint-plugin-no-mistakes` | Keep Playwright test IDs, fetch calls, exports, function wrappers, and ReactNode fallbacks statically analyzable. |

## Install

Use the published packages where available:

```sh
npm install --save-dev no-mistakes eslint-plugin-no-mistakes
```

Or install the Rust binary directly:

```sh
cargo install no-mistakes
```

For local development from a clone, run workspace binaries with Cargo:

```sh
cargo run -p no-mistakes -- dependents src/utils.mts --format paths
```

## Link Lint

Documentation links are linted with [lychee](https://github.com/lycheeverse/lychee):

```sh
lychee --no-progress --exclude-path '^fixtures/' README.md 'docs/**/*.md' 'skills/**/*.md' 'packages/*/README.md' 'crates/*/README.md' CLAUDE.md
```
