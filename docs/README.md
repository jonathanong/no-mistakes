# Documentation

`no-mistakes` parses the current checkout into one in-memory graph, then
projects that graph into impact queries, test plans, and checks. Start with
the job you have; the rest of this index is reference.

## Start Here

| Audience | Goal | Doc |
| --- | --- | --- |
| Everyone | Why it exists, and which agent mistakes it prevents | [Why no-mistakes exists](why.md) |
| Humans | Install, first queries, and what config unlocks | [README](../README.md#install-and-first-queries), [Configuration](configuration/README.md) |
| Humans | Repository checks and file-local lint rules | [no-mistakes rules](rules/README.md), [ESLint rules](eslint-rules/README.md) |
| Agents | Change lifecycle and command selection | [Agent guide](agent-guide.md) |
| Agents | Compact skill to install in a harness | [Packaged agent skill](../skills/no-mistakes/SKILL.md) |
| Either | Pick a CLI command or async Node API | [CLI commands](cli/README.md), [Node/N-API guide](node-api.md) |
| Either | Diagnose analysis cost without changing output | [Performance diagnostics](cli/diagnostics.md) |
| Either | Static-analysis limits | [AST analysis behavior](ast-analysis.md) |
| Either | Language and framework coverage contract | [Feature parity](feature-parity.md) |

## Reference

- [PostgreSQL facts](postgres-facts.md) describes the schema and embedded-SQL fact sources used by later SQL rules.
- [Architecture](architecture.md) describes the one-pass, in-memory, graph-based
  execution model.
- [AST-grep regression rules](ast-grep-rules.md) guard the source shapes that
  would bypass the one-pass gateways.
- [Graph edges](graph-edges.md) lists dependency edge kinds with fixture-backed
  examples, counterexamples, relationship filters, and caveats.
- [Feature parity](feature-parity.md) is the language-frontend contract
  relative to the TS/JS surface (Python, Django, Celery, Go, Asynq, Kafka,
  Rust, Rails, PHP, Java, Kotlin, Elixir, Dart, Swift, and .NET).
- [Test planning](test-plan.md) explains configured test selection in more
  depth.
- [Legacy CLI reference](cli-reference.md) and [legacy ESLint reference](eslint-plugin.md)
  remain compatibility landing pages that point to the split docs.

## Validation

```sh
lychee --no-progress --exclude-path '^fixtures/' README.md 'docs/**/*.md' 'skills/**/*.md' 'packages/*/README.md' 'crates/*/README.md' CLAUDE.md
```
