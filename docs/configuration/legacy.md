# Legacy Config Migration

Unified `.no-mistakes.yml` is preferred.

Legacy config stems are still read for compatibility:

- `.playwright-ast-coverage.*`
- `.react-traits.*`
- `.next-to-fetch.*`
- `.guardrailsrc.{yaml,yml,json,jsonc}` discovered by walking upward from the
  requested root
- `guardrailsrc.{yaml,yml,json,jsonc}` only when passed explicitly through
  `--config`

When migrating, move per-tool settings into `tests`, `projects`, `testPlan`,
and `rules`. Prefer `testPlan.<framework>.fullSuiteTriggers` over the
deprecated `dependencies` key.
