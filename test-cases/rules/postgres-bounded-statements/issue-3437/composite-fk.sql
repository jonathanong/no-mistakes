-- membership.id fixes source and lineage. The composite foreign key then fixes context_id.
SELECT operation.id
FROM memberships membership
JOIN operations operation ON operation.source_id = membership.source_id
JOIN sources source ON source.id = operation.source_id
JOIN lineages lineage ON lineage.id = operation.lineage_id
WHERE membership.id = $1 AND operation.kind = 'administrator_refund' AND operation.idempotency_key = $2
  AND source.lineage_id = lineage.id AND operation.lineage_id = lineage.id;

-- A nullable context_id does not take the referenced row's value.
SELECT operation.id
FROM memberships membership
JOIN nullable_operations operation ON operation.source_id = membership.source_id
JOIN sources source ON source.id = operation.source_id
JOIN lineages lineage ON lineage.id = operation.lineage_id
WHERE membership.id = $1 AND operation.kind = 'administrator_refund' AND operation.idempotency_key = $2
  AND source.lineage_id = lineage.id AND operation.lineage_id = lineage.id;

-- The foreign key targets a column set that is not a unique key.
SELECT operation.id
FROM memberships membership
JOIN loose_operations operation ON operation.source_id = membership.source_id
JOIN sources source ON source.id = operation.source_id
JOIN tag_pairs tag ON tag.id = operation.lineage_id
WHERE membership.id = $1 AND operation.kind = 'administrator_refund' AND operation.idempotency_key = $2
  AND source.lineage_id = tag.id AND operation.lineage_id = tag.id;

-- lineage_id stays unpinned when membership.id is not fixed.
SELECT operation.id
FROM memberships membership
JOIN operations operation ON operation.source_id = membership.source_id
JOIN sources source ON source.id = operation.source_id
JOIN lineages lineage ON lineage.id = operation.lineage_id
WHERE operation.kind = 'administrator_refund' AND operation.idempotency_key = $2
  AND source.lineage_id = lineage.id AND operation.lineage_id = lineage.id;
