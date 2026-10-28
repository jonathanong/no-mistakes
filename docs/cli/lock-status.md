# lock status

Report whether another `no-mistakes` invocation currently holds the
machine-wide invocation lock, and who. Use this to tell a busy lock from a hang
before running `check`.

## Usage

```bash
no-mistakes lock status [--format text|json]
```

## Options

| Flag | Description |
|------|-------------|
| `--format` | `text` (default) prints `free` or `held by pid ...`; `json` prints one object. |

`lock status` is read-only and never waits: it does not take the invocation
lock, so it answers immediately even while another invocation holds it, and it
ignores `--lock-timeout` and `--fail-on-lock`. It probes the lock with an
instantaneous shared lock and never rewrites the holder record. Exit status is
`0` whether the lock is held or free.

## JSON output

```json
{"state":"held","lockFile":"/home/me/.cache/no-mistakes/invocation.lock","holder":{"pid":264,"cwd":"/work/repo","command":"no-mistakes check --format json","startedAtMs":1700000000000,"ageSeconds":12}}
```

- `state` is `held` or `free`.
- `holder` is `null` when the lock is free (a leftover record from an exited
  process is never reported) or when the lock is held but its record is empty or
  unreadable. A holder that predates holder records is reported with only `pid`
  set; `cwd` is `null` and `command` is empty.
- `ageSeconds` is computed when read; it is `null` when the start time is
  unknown.

Node API: `lockStatus()` returns the same object. See
[Node API](../node-api.md).
