---
name: integrations-rules
description: "MUST NOT / SHOULD NOT rules for third-party APIs and inbound webhooks: response classification, retries and idempotency, delivery records, payload mapping, timestamps and units, batch failure policy, and signature verification. Use when calling, parsing, or receiving a partner or vendor API."
---

# Integration and webhook rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

## Outbound calls

- MUST NOT call a partner without separate connect and read timeouts; a missing read timeout hangs the worker.
- MUST NOT retry a non-idempotent call without an idempotency key the provider honors, or a lookup proving the first attempt failed.
- MUST NOT record "sent" before delivery is confirmed.
- MUST NOT round money, weights, or quantities before the final step, or mix units.
- MUST NOT report a batch done when any item failed; record each item's result.
- SHOULD NOT treat a 2xx as proof of success; check the body for an error.
- SHOULD NOT let a partner's payload shape pass the mapping layer; map it to your own types once.
- SHOULD NOT index into a partner payload as if it were your schema; tolerate missing, null, and extra fields.
- SHOULD NOT store partner timestamps without normalizing them to UTC at the boundary.
- SHOULD NOT leave the batch failure policy implicit; choose per-item or whole-batch and say which.
- SHOULD NOT test the boundary only with hand-written payloads; replay captured responses.

## Inbound webhooks

- MUST NOT parse, persist, or enqueue before verifying the signature against the raw body.
- MUST NOT return 2xx before the durable handoff succeeds: an enqueue, or an idempotency row that records the pending work so a crash cannot lose it.
- MUST NOT assume a webhook arrives once or in order; dedupe by event id and compare versions.
- SHOULD NOT discard the raw body; store it with a TTL and access limits so failures can be replayed.
