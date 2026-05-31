# `no-mistakes queues`

Analyze queue producer/worker relationships for BullMQ and glide-mq patterns.

| Leaf command | Purpose |
| --- | --- |
| [`queues edges`](queues-edges.md) | Print queue dependency edges. |
| [`queues related`](queues-related.md) | Print files/nodes related to queue files or jobs. |
| [`queues check`](queues-check.md) | Fail on unmatched producers/workers. |

Shared options: `--root`, `--tsconfig`, repeatable `--filter`, `--depth`,
`--format`, `--json`, and `--timings`.
