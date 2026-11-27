# lock status

Report whether another `no-mistakes` invocation currently holds the
per-user invocation lock, and who. Use this to tell a busy lock from a hang
before running `check`.

## Usage

```bash
no-mistakes lock status [--format text|json]
```

## Options

| Flag | Description |
|------|-------------|
| `--format` | `text` (default) prints `free`, `held by <description>`, or `held by another no-mistakes invocation (holder details unavailable)` when the lock is held but the record cannot be read. It also prints the selected lock file. Without a usable location, it prints `unavailable (no invocation lock selected)`. `json` prints one object. |

`lock status` never waits. It can create the lock directory and an empty lock
file while selecting a location. It probes the file with an instantaneous
shared lock and leaves the holder record untouched. That brief
shared probe can contend with a simultaneous `--fail-on-lock` acquisition, so
a fail-fast invocation may report the lock as busy during the probe. The
command ignores `--timeout`, `--invocation-timeout-ms`, `--lock-timeout`, and
`--fail-on-lock`. Exit status is `0` when the lock is held, free, or unavailable.

## Lock locations

Invocations and status try these directories in order. Each uses the first
location where it can open `invocation.lock` for writing, creating missing
directories as needed.

1. The absolute directory in `NO_MISTAKES_LOCK_DIR`, when set.
2. The per-user runtime directory for no-mistakes, when available.
3. The per-user cache directory for no-mistakes.
4. On Unix, a private `no-mistakes-<uid>` directory inside the process
   temporary directory (from `TMPDIR` when set), then inside `/tmp`. On
   Windows, `no-mistakes` under the user's local application data `Temp`
   directory.

Runs that select the same lock file serialize across processes. To share a lock
across environments, point the absolute override at a directory that is shared
and writable in each environment. Run `lock status --format json` in each
environment and verify that `lockFile` identifies the expected shared file.

If any candidate fails, the command prints one JSON warning to stderr. Its
`kind` is `invocation_lock_fallback`, `failed` lists the attempted paths and
errors, and `lockFile` names the selected path. If every candidate fails,
`lockFile` is `null` and analysis proceeds without an invocation lock.
Successful analysis reports remain on stdout. The Node API still serializes
analyses within each Node process to protect its shared deadline state.

## JSON output

```json
{"state":"held","lockFile":"/home/me/.cache/no-mistakes/invocation.lock","holder":{"pid":264,"cwd":"/work/repo","command":"no-mistakes","startedAtMs":1700000000000,"ageSeconds":12}}
```

- `lockFile` is the selected path or `null` when no location is usable.
- `state` is `held`, `free`, or `unavailable`.
- `holder` is `null` when the lock is unavailable, when it is free (a leftover record from an exited
  process is never reported) or when the lock is held but its record is empty or
  unreadable. The sidecar is best-effort: it remains after normal release or a
  crash, and can briefly describe the previous holder while a new holder is
  acquiring the lock. New records contain the executable basename in `command`
  and omit arguments to avoid persisting option or positional values. Older
  records may contain a full command line. A holder that predates holder records
  is reported with only `pid` set; `cwd` is `null` and `command` is empty.
- Holder records are replaced atomically using an owner-only temporary file
  (`0600` on Unix). A process killed while writing can leave `.tmp*` staging
  files in the lock directory. They are not holder records, do not affect lock
  acquisition or status, and can remain until manually removed.
- `ageSeconds` is computed when read; it is `null` when the start time is
  unknown.

Node API: `lockStatus()` returns the same object. See
[Node API](../node-api.md).
