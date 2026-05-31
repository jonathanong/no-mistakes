# `no-mistakes queues check`

Check for unmatched queue producers and workers.

```sh
no-mistakes queues check --format json
```

Use this before finishing queue edits so every configured enqueue/worker path is
connected.

Node API: `queueCheck(options)`.
