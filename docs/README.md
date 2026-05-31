# Documentation

`no-mistakes` provides local, deterministic AST tools for codebase
intelligence. The docs are organized around agent tasks: choose a command,
request structured output, understand config, and keep source code analyzable.

## Start Here

| Goal | Doc |
| --- | --- |
| Pick the right CLI command | [CLI commands](cli/README.md) |
| Call the async Node API | [Node/N-API guide](node-api.md) |
| Configure projects, tests, selectors, and rules | [Configuration](configuration/README.md) |
| Configure repository checks | [no-mistakes rules](rules/README.md) |
| Keep file-local code analyzable | [ESLint rules](eslint-rules/README.md) |
| Understand static-analysis limits | [AST analysis behavior](ast-analysis.md) |
| Use the tool as an AI agent | [Agent guide](agent-guide.md) |

## Reference

- [Architecture](architecture.md) describes the one-pass, in-memory, graph-based
  execution model.
- [Graph edges](graph-edges.md) lists dependency edge kinds with fixture-backed
  examples, counterexamples, relationship filters, and caveats.
- [Test planning](test-plan.md) explains configured test selection in more
  depth.
- [Legacy CLI reference](cli-reference.md) and [legacy ESLint reference](eslint-plugin.md)
  remain compatibility landing pages that point to the split docs.

## Validation

```sh
lychee --no-progress --exclude-path '^fixtures/' README.md 'docs/**/*.md' 'skills/**/*.md' 'packages/*/README.md' 'crates/*/README.md' CLAUDE.md
```
