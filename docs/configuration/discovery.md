# Config Discovery

Discovery order:

1. `--config <path>` when provided.
2. `.no-mistakes.{yaml,yml,json,jsonc}` under `--root`.
3. Legacy per-tool config stems under `--root`.
4. legacy guardrails config files walking upward from `--root`.
5. Empty default config.

If multiple files with the same discovered stem exist under `--root`, loading
fails so agents do not guess which config is authoritative.
