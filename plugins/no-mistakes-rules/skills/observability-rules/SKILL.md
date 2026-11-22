---
name: observability-rules
description: "Load before adding or reviewing a log line, a metric, an alert, or a monitor (pino, winston, OpenTelemetry, Datadog, Sentry), or making claims about production behavior or reporting measured results. Lists the defects to avoid and the patterns that need a stated reason: personal data, metric cardinality, fields monitors depend on, absence alerts, bound context, metrics clients across forks, evidence for claims."
---

# Observability rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

- MUST NOT tag a metric with unbounded values (ids, emails, URLs); use a closed set, and apply the personal-data rule below when logging high-cardinality values.
- MUST NOT rename or retype a log field or metric that a monitor or dashboard reads without updating it in the same change.
- MUST NOT let transaction-specific log context leak into unrelated work after rollback. Clear or scope the context when the logger retains it across rollback.
- MUST NOT create or first use a metrics client before a process forks; create it per process after the fork.
- SHOULD NOT log personal data such as emails, names, postal addresses, or phone numbers in messages, structured fields, bound context, or payloads; omit or redact it, or use a stable pseudonymous identifier when needed for correlation. Pseudonymous identifiers remain linkable. Exceptions for necessary audit or security records need a documented purpose, minimal fields, restricted access, and a defined retention period.
- SHOULD NOT alert only on errors for scheduled or queued work; also alert when expected work stops.

## Evidence

- MUST NOT treat an empty result from an unproven query as absence; first show the query can return rows.
- MUST NOT state a rate without its denominator.
- MUST NOT treat a capped page of results as the whole population.
- MUST NOT repeat a number from a subagent's or tool's prose summary; read the raw output.
- MUST NOT call a fix verified because a lower environment shows no error.
- MUST NOT treat "rare in production" as "unreachable"; they need different proof.
