---
name: rollout-rules
description: "MUST NOT / SHOULD NOT rules for shipping changes safely: feature flags, deploy order across services, API and message compatibility, irreversible actions, and string-referenced config. Use when adding a flag, changing an API or message contract, deploying a change that spans services, or touching money-moving or irreversible actions."
---

# Rollout rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

## Flags

- MUST NOT ship a new feature flag with the new behavior active by default.
- MUST NOT flag some paths and not others; every entry point that reaches the new behavior reads the flag.
- MUST NOT assume flag-off is a no-op; test the off path, and handle work started under the other value.
- MUST NOT assume a flag flip is instant in a service that caches it.
- SHOULD NOT rely on a flag that has never been toggled in that environment; toggle it there and watch the effect first.
- SHOULD NOT leave a fully rolled-out flag in code; remove it and its off path.

## Deploys and contracts

- MUST NOT deploy a producer of a new shape before every consumer accepts it; expand, migrate, then contract.
- MUST NOT fail open on irreversible or money-moving actions; when unsure, stop and report.
- SHOULD NOT conflate deployed with active; ship dark, then enable.
- SHOULD NOT reference config, task names, or handlers by string without validating them at startup.
- SHOULD NOT leave an implicit contract between services untested; add a contract test on the side you own.
