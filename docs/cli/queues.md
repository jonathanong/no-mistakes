# `no-mistakes queues`

Analyze queue producer/worker relationships for BullMQ, glide-mq, Celery,
Asynq, Kafka topics, Active Job, and Laravel `dispatch`.

| Leaf command | Purpose |
| --- | --- |
| [`queues edges`](queues-edges.md) | Print queue dependency edges. |
| [`queues related`](queues-related.md) | Print files/nodes related to queue files or jobs. |
| [`queues check`](queues-check.md) | Fail on unmatched producers/workers. |

Shared options: `--root`, `--tsconfig`, repeatable `--filter`, `--depth`,
`--format`, `--json`, and `--timings`.
