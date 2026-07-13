# Config Discovery

Discovery order:

1. `--config <path>` when provided.
2. `.no-mistakes.{yaml,yml,json,jsonc}` under `--root`.
3. Legacy per-tool config stems under `--root`.
4. legacy guardrails config files walking upward from `--root`.
5. Empty default config.

If multiple files with the same discovered stem exist under `--root`, loading
fails so agents do not guess which config is authoritative.

## Git visibility

Automatic file discovery uses Git's visible file set: tracked files plus
untracked files not excluded by `.gitignore`, `.git/info/exclude`, or global
Git excludes. Tracked files remain visible even when they match an ignore
pattern. Outside a Git checkout, `.gitignore` and `.ignore` files are still
applied by the fallback walker.

Explicit paths supplied through CLI flags or configuration remain authoritative
and may name an ignored file. This exception applies to explicit configuration,
not to automatically discovered source, test, workflow, or runner-config files.
