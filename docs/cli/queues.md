# `no-mistakes queues`

Analyze queue producer/worker relationships for configured queue frameworks.

Configure custom TypeScript factories and language/project queue globs in
[Queue configuration](../configuration/queues.md) before running these
queries. Queue analysis is opt-in and scoped; it does not infer repository-wide
conventions.

Configured Celery, Asynq, Kafka, Active Job, Sidekiq, Laravel, and Symfony Messenger
sites emit the same canonical `queue-enqueue` / `queue-worker` graph edges.
`queues edges|related|check` projects those language edges from the same
facts as the dependency graph. TypeScript baseline fields stay identical
when language packages are configured on a TypeScript-only tree.

| Leaf command | Purpose |
| --- | --- |
| [`queues edges`](queues-edges.md) | Print queue dependency edges. |
| [`queues related`](queues-related.md) | Print files/nodes related to queue files or jobs. |
| [`queues check`](queues-check.md) | Report unmatched producers and workers, cycles, self-dedup enqueues, shared-queue rate limits, enqueue waits, invalid custom job ids, job-scheduler template options, and mixed priorities. |

Shared options: `--root`, `--tsconfig`, repeatable `--filter`, `--depth`
(alias `--max-depth`; with roots and no depth, edges default to depth 1, and
with no roots an omitted depth prints every edge), `--format` (default
`human`), `--json`, and `--timings`.
