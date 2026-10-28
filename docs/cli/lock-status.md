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

`lock status` is read-only and never waits. It probes the lock with an
instantaneous shared lock and never rewrites the holder record. That brief
shared probe can contend with a simultaneous `--fail-on-lock` acquisition, so
a fail-fast invocation may report the lock as busy during the probe. The
command ignores `--timeout`, `--invocation-timeout-ms`, `--lock-timeout`, and
`--fail-on-lock`. Exit status is `0` whether the lock is held or free.

## JSON output

```json
{"state":"held","lockFile":"/home/me/.cache/no-mistakes/invocation.lock","holder":{"pid":264,"cwd":"/work/repo","command":"no-mistakes","startedAtMs":1700000000000,"ageSeconds":12}}
```

- `state` is `held` or `free`.
- `holder` is `null` when the lock is free (a leftover record from an exited
  process is never reported) or when the lock is held but its record is empty or
  unreadable. The sidecar is best-effort: it remains after normal release or a
  crash, and can briefly describe the previous holder while a new holder is
  acquiring the lock. New records contain the executable basename in `command`
  and omit arguments to avoid persisting option or positional values. Older
  records may contain a full command line. A holder that predates holder records
  is reported with only `pid` set; `cwd` is `null` and `command` is empty.
- `ageSeconds` is computed when read; it is `null` when the start time is
  unknown.

Node API: `lockStatus()` returns the same object. See
[Node API](../node-api.md).
