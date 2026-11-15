-- membership.id fixes source, lineage, and context_id; the idempotency key then matches one operation.
SELECT operation.id FROM memberships membership
JOIN operations operation ON operation.source_id = membership.source_id
JOIN sources source ON source.id = operation.source_id
JOIN lineages lineage ON lineage.id = operation.lineage_id
WHERE membership.id = $1 AND operation.operation_kind = 'refund'
  AND operation.idempotency_key = $2 AND source.lineage_id = lineage.id
  AND operation.lineage_id = lineage.id;

-- The composite unique key still has idempotency_key free.
SELECT operation.id FROM memberships membership
JOIN operations operation ON operation.source_id = membership.source_id
JOIN sources source ON source.id = operation.source_id
JOIN lineages lineage ON lineage.id = operation.lineage_id
WHERE membership.id = $1 AND operation.operation_kind = 'refund'
  AND source.lineage_id = lineage.id
  AND operation.lineage_id = lineage.id;

-- A non-unique source_id join does not bound operations.
SELECT operation.id FROM memberships membership
JOIN operations operation ON operation.source_id = membership.source_id
WHERE membership.id = $1;

-- A NOT VALID foreign key does not fill context_id.
SELECT operation.id FROM memberships membership
JOIN pending_operations operation ON operation.source_id = membership.source_id
JOIN sources source ON source.id = operation.source_id
JOIN lineages lineage ON lineage.id = operation.lineage_id
WHERE membership.id = $1 AND operation.operation_kind = 'refund'
  AND operation.idempotency_key = $2 AND source.lineage_id = lineage.id
  AND operation.lineage_id = lineage.id;

-- A nullable foreign-key column does not fill the composite key.
SELECT operation.id FROM memberships membership
JOIN draft_operations operation ON operation.source_id = membership.source_id
JOIN sources source ON source.id = operation.source_id
JOIN lineages lineage ON lineage.id = operation.lineage_id
WHERE membership.id = $1 AND operation.operation_kind = 'refund'
  AND operation.idempotency_key = $2 AND source.lineage_id = lineage.id
  AND operation.lineage_id = lineage.id;
