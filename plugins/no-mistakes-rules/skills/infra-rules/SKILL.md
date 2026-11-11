---
name: infra-rules
description: "MUST NOT / SHOULD NOT rules for infrastructure and ops scripts: OpenTofu or Terraform, IAM, secrets stores, provisioning and cleanup scripts, and shell. Use when writing or reviewing infrastructure code, machine setup, or shell scripts."
---

# Infrastructure and shell rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

## Infrastructure as code

- MUST NOT apply, force-unlock, or mutate live infrastructure without authorization for that environment and a reviewed saved plan.
- MUST NOT post sensitive plan output in PRs or shared notes.
- MUST NOT manage long-lived IAM users in code; use roles and trust policies.
- MUST NOT use IAM wildcard resources without a reason beside the statement.
- MUST NOT rely on generated names that other systems reference; set stable names.
- MUST NOT put browser-visible values in secret stores.
- SHOULD NOT put topology (counts, scaling, resource ids) in application repos.
- SHOULD NOT rely on alerts and runbooks where automatic detection and scoped remediation are possible.

## Scripts and shell

- MUST NOT make destructive scripts act by default; report first, then require `--apply`.
- MUST NOT write non-idempotent provisioning; detect before acting and support `--dry-run`.
- MUST NOT hardcode owners, repos, host names, or labels; take parameters.
- MUST NOT persist registration tokens or read credential files; fetch at runtime and use immediately.
- MUST NOT append duplicate rc lines on rerun; guard with `grep -qxF`.
- MUST NOT ship shell scripts that fail ShellCheck.
- MUST NOT assign shell read-only names such as zsh `status`; expand arrays safely under `set -u`.
- SHOULD NOT write shell scripts longer than about 300 lines; split them into steps.
