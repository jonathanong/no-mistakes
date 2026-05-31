# `no-mistakes queues edges`

Print queue dependency edges.

```sh
no-mistakes queues edges src/jobs/enqueue.mts --format json
```

With no roots, prints all queue edges. With roots and no explicit depth, prints
direct edges only.

Node API: `queueEdges(options)`.
