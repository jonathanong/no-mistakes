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
pattern. The request snapshot also retains a tracked-only view for
repository-state rules such as `banned-paths`; that view contains files present
in both the Git index and working tree, including tracked files that now match
an ignore pattern, but excludes all untracked files. Both views come from the
same Git discovery command and are reused throughout the request.

Outside a Git checkout, `.gitignore` and `.ignore` files are still applied by
the fallback walker. Because there is no Git index, rules that normally use the
tracked-only view use this ignore-aware visible set instead.

Source, dependency-graph, and test discovery derive narrower views from that
inventory and prune built-in source skip directories such as `fixtures`,
`build`, `dist`, and `target`. Repository-state policies such as a
repository-scoped `banned-paths` rule consume the repository inventory instead,
so putting a tracked artifact below a source skip directory does not exempt it
from the policy.

Explicit paths supplied through CLI flags or configuration remain authoritative
and may name an ignored file. This exception applies to explicit configuration,
not to automatically discovered source, test, workflow, or runner-config files.
