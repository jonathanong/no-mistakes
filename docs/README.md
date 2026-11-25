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
the full documentation set. The gateway wrapper retries the selected documents using
the `LYCHEE_GATEWAY_WAIT` schedule and the initial check’s GitHub API credentials
in [the workflow](../.github/workflows/ci.yml). After those attempts, a fresh report
containing only HTTP 5xx responses for links to `github.com` is inconclusive:
CI emits warnings with the affected URLs rather than treating a server error as
proof of a broken link. The final report remains in the job summary for diagnosis.

The selector tests require the same `yq` v4 executable used on the CI runner.

HTTP 404 and other client errors, failures from other hosts, timeouts, incomplete
reports (including an unterminated final entry), and checker execution errors
still fail the gate. A report containing a GitHub server error alongside any such
failure cannot pass. The wrapper removes
the previous report before each attempt so a checker crash cannot reuse stale
results. It does not turn all link failures into accepted status codes or rerun
CI jobs.
