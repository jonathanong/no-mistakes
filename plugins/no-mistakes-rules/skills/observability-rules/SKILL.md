---
name: observability-rules
description: "MUST NOT / SHOULD NOT rules for logs, metrics, and alerts: static messages and structured fields, personal data, metric cardinality, fields monitors depend on, absence alerts, and context binding. Use when adding or reviewing logging, metrics, or monitors."
---

# Observability rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

- MUST NOT tag a metric with unbounded values (ids, emails, URLs); use a closed set, and apply the personal-data rule below when logging high-cardinality values.
- MUST NOT rename or retype a log field or metric that a monitor or dashboard reads without updating it in the same change.
- MUST NOT assume context bound inside a transaction is cleared when the transaction rolls back.
- MUST NOT create or first use a metrics client before a process forks; create it per process after the fork.
- SHOULD NOT log personal data such as emails, names, postal addresses, or phone numbers in messages, structured fields, bound context, or payloads; omit or redact it, or use a stable pseudonymous identifier when needed for correlation. Pseudonymous identifiers remain linkable. Exceptions for necessary audit or security records need a documented purpose, minimal fields, restricted access, and a defined retention period.
- SHOULD NOT interpolate dynamic values into a log message; keep the message static and put values in structured fields.
- SHOULD NOT alert only on errors for scheduled or queued work; also alert when expected work stops.
- SHOULD NOT load log context by walking relations per line; fetch it with the query.
