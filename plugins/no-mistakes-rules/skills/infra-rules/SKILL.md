---
name: infra-rules
description: "Load before writing or reviewing infrastructure and ops code: OpenTofu or Terraform, IAM policies, secrets stores, provisioning or cleanup scripts, shell scripts, health probes, or code whose correctness depends on how many instances run. Lists the defects to avoid and the patterns that need a stated reason."
---

# Infrastructure and shell rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

## Infrastructure as code

- MUST NOT apply, force-unlock, or mutate live infrastructure without authorization for that environment.
- SHOULD apply a reviewed saved plan when changes require plan review.
- MUST NOT post sensitive plan output in PRs or shared notes.
- SHOULD NOT provision long-lived IAM users when the workload supports temporary credentials.
- MUST NOT use IAM wildcard resources without a reason beside the statement.
- SHOULD NOT make other systems depend on generated names that can change without updating those systems. Publish a stable identifier or update references with the resource.
- SHOULD NOT store browser-visible configuration in a secret store unless the deployment needs it there. Treat values delivered to the browser as public.

## Runtime

- MUST NOT run side-effecting scheduled work from an in-process timer in a service with more than one replica; use a scheduler that runs it once.
- MUST NOT make a liveness probe check a shared dependency; one outage restarts every instance.
- SHOULD NOT keep state that request correctness depends on (sessions, uploads, counters, locks) in process memory or on local disk; the next request may land on another instance.
- SHOULD NOT open a database connection per request; pool, and size the pool against the database limit across all instances.

## Scripts and shell

- SHOULD NOT make destructive scripts act by default on persistent resources. Provide a preview before applying changes.
- SHOULD NOT make provisioning unsafe to retry after a partial failure.
- MUST NOT persist registration tokens or read credential files; fetch at runtime and use immediately.
- MUST NOT append duplicate shell startup lines on rerun.
- SHOULD NOT ship shell scripts with unresolved ShellCheck findings. Document any justified suppression.
- MUST NOT assign shell read-only names such as zsh `status`; expand arrays safely under `set -u`.
