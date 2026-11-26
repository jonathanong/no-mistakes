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
| Agents | Compact skill to install in a harness | [Packaged agent skill](../plugins/no-mistakes/skills/no-mistakes/SKILL.md) |
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

`docs/performance/2238/39488578982/README.md` and `tables.md` stay byte-for-byte with `checksums.json`. The check skips those two files because their recorded Actions URL is no longer served. The other matched documents stay in the check.

```sh
lychee --no-progress --exclude-path '^fixtures/' --exclude-path 'docs/performance/2238/39488578982/(README|tables)\.md$' README.md 'docs/**/*.md' 'plugins/*/skills/**/*.md' 'packages/*/README.md' 'crates/*/README.md' AGENTS.md
```

Pull requests normally check links in changed documentation files. When the
online scan is scoped to changed files, a deletion or rename adds an offline
check across all documentation to find broken links in unchanged pages.
Changes to the link-check configuration, including Lychee arguments such as
exclusions and remaps, trigger a full scan. Pushes and manual runs also check
the full documentation set.

The [gateway wrapper](../.github/scripts/lychee-gateway.cjs) preserves the
Lychee report and warns immediately when it contains the reported GitHub
HTTP/2 protocol error, every other error is a GitHub HTTP 5xx response or the
same protocol error, and there are zero timeouts. The warning lists the URLs
whose validity remains unknown. It uses the existing bounded retry schedule
for reports containing only retryable HTTP responses. A fresh report with only
GitHub HTTP 5xx errors also warns once those attempts are exhausted.

The selector tests require the same `yq` v4 executable used on the CI runner.

HTTP 404 and other client errors, persisting failures from other hosts,
timeouts, incomplete reports, and checker execution errors still fail the
gate. The wrapper removes the previous report before each existing retry so a
checker crash cannot reuse stale results. The final report stays in the job
summary.
