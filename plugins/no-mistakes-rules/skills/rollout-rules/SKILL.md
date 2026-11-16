---
name: rollout-rules
description: "MUST NOT / SHOULD NOT rules for shipping changes safely: feature flags, deploy order across services, API and message compatibility, unreachable new code, irreversible actions, and string-referenced config. Use when adding a flag, a table, a service, or a write path, changing an API or message contract, deploying a change that spans services, or touching money-moving or irreversible actions."
---

# Rollout rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

## Flags

- MUST NOT flag some paths and not others; every entry point that reaches the new behavior reads the flag.
- MUST NOT assume flag-off is a no-op; test the off path, and handle work started under the other value.
- MUST NOT assume a flag flip is instant in a service that caches it.
- SHOULD NOT rely on a flag that has never been toggled in that environment; toggle it there and watch the effect first.
- SHOULD NOT leave a fully rolled-out flag in code; remove it and its off path.

## Deploys and contracts

- MUST NOT deploy a producer of a new shape while consumers that cannot read it can still run; expand, migrate, then contract.
- MUST NOT add a table, service, or write path that no entry point (API route, MCP tool, CLI command, or job) reaches, directly or through code it calls; build the entry point in the same change, even if nothing calls it yet. Exceptions: a schema expand step that deploys ahead of the code that uses it, and a library whose entry points live in its consumers' repos.
- MUST NOT fail open on irreversible or money-moving actions; when unsure, stop and report.
- SHOULD NOT reference config, task names, or handlers by string without validating them at startup.
