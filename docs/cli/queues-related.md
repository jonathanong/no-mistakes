# `no-mistakes queues related`

Print queue files and virtual job nodes related to inputs.

```sh
no-mistakes queues related src/jobs/enqueue.mts --direction both --format paths
```

Use this to connect producers, queue definitions, processors, and workers.

Key option: `--direction deps|dependents|both`.

Node API: `queueRelated(options)`.
