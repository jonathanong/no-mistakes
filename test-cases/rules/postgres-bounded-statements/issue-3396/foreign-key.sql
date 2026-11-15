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

-- A CTE is not the referenced table, so the foreign key does not fill the key.
WITH other AS (SELECT id, context_id FROM lineages)
SELECT operation.id FROM other
JOIN operations operation
  ON operation.lineage_id = other.id AND operation.context_id = other.context_id
WHERE operation.idempotency_key = $2;

-- An unknown table does not supply the referenced row.
SELECT operation.id FROM operations operation
JOIN missing_lineages lineage
  ON operation.lineage_id = lineage.id AND operation.context_id = lineage.context_id
WHERE operation.idempotency_key = $2;

-- A nullable foreign-key column does not fill the composite key.
SELECT operation.id FROM memberships membership
JOIN draft_operations operation ON operation.source_id = membership.source_id
JOIN sources source ON source.id = operation.source_id
JOIN lineages lineage ON lineage.id = operation.lineage_id
WHERE membership.id = $1 AND operation.operation_kind = 'refund'
  AND operation.idempotency_key = $2 AND source.lineage_id = lineage.id
  AND operation.lineage_id = lineage.id;

-- A positional alias that reuses a foreign-key name is not that physical column.
SELECT o.id
FROM operations AS o(id, lineage_id, source_id, context_id, operation_kind, idempotency_key)
JOIN lineages l ON o.lineage_id = l.id
WHERE l.id = $1 AND o.idempotency_key = $2;

-- The same alias list in catalog order still completes the composite key.
SELECT o.id
FROM operations AS o(id, source_id, lineage_id, context_id, operation_kind, idempotency_key)
JOIN lineages l ON o.lineage_id = l.id AND o.context_id = l.context_id
WHERE l.id = $1 AND l.context_id = $2 AND o.idempotency_key = $3;

-- A complete alias list still names each foreign-key column. The join does not equate ctx.
SELECT o.op_id
FROM operations AS o(op_id, src, lin, ctx, kind, idem)
JOIN lineages l ON o.lin = l.id
WHERE l.id = $1 AND l.context_id = $2 AND o.idem = $3;

-- A repeated alias does not identify one foreign-key column.
SELECT o.id
FROM operations AS o(id, source_id, lineage_id, lineage_id, operation_kind, idempotency_key)
JOIN lineages l ON o.lineage_id = l.id AND o.context_id = l.context_id
WHERE l.id = $1 AND l.context_id = $2 AND o.idempotency_key = $3;

-- The preserved side stays unbounded when the driver is only on the right of a left join.
SELECT operation.id
FROM operations operation
LEFT JOIN memberships membership
  ON operation.source_id = membership.source_id AND membership.id = $1;

-- OR inside the join condition does not pin the unique key.
SELECT operation.id FROM memberships membership
JOIN operations operation ON operation.source_id = membership.source_id OR operation.id = membership.id
WHERE membership.id = $1 AND operation.idempotency_key = $2;

-- A partial unique index does not make the referenced id one row.
SELECT operation.id
FROM partial_lineages lineage
JOIN partial_operations operation
  ON operation.lineage_id = lineage.id AND operation.context_id = lineage.context_id
WHERE lineage.id = $1 AND operation.idempotency_key = $2;

-- Columns past a short alias list keep their names, so the foreign key still fills context_id.
SELECT o.op_id
FROM memberships membership
JOIN operations AS o(op_id) ON o.source_id = membership.source_id
JOIN sources source ON source.id = o.source_id
JOIN lineages lineage ON lineage.id = o.lineage_id
WHERE membership.id = $1 AND o.operation_kind = 'refund'
  AND o.idempotency_key = $2 AND source.lineage_id = lineage.id
  AND o.lineage_id = lineage.id;

-- The primary key sits past the alias list and keeps its name.
SELECT row.id FROM labeled_keys AS row(title) WHERE row.id = $1;

-- The alias reuses id, so that name is not the later primary key.
SELECT row.title FROM labeled_keys AS row(id) WHERE row.id = $1;
