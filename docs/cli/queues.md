# `no-mistakes queues`

Analyze queue producer/worker relationships for BullMQ and glide-mq.

Configured Celery, Asynq, Kafka, Active Job, and Laravel sites emit the same
canonical `queue-enqueue` / `queue-worker` graph edges. Query those through
`dependents --relationship queue` (or the language filter). The dedicated
`queues` command still reports the TypeScript queue pipeline; wiring those
language edges into `queues edges|related|check` is later work.

| Leaf command | Purpose |
| --- | --- |
| [`queues edges`](queues-edges.md) | Print queue dependency edges. |
| [`queues related`](queues-related.md) | Print files/nodes related to queue files or jobs. |
| [`queues check`](queues-check.md) | Fail on unmatched producers/workers. |

Shared options: `--root`, `--tsconfig`, repeatable `--filter`, `--depth`,
`--format`, `--json`, and `--timings`.
