# `no-mistakes queues edges`

Print queue dependency edges, including configured language
`queue-enqueue` / `queue-worker` rows.

```sh
no-mistakes queues edges src/jobs/enqueue.mts --format json
```

With no roots, prints all queue edges. With roots and no explicit depth, prints
direct edges only.

Kafka topic edges from the literals in [feature parity](../feature-parity.md#kafka)
use this command. The job identity is `<cluster>:<topic>` when a project
cluster is set, and the bare topic otherwise.

Node API: `queueEdges(options)`.
