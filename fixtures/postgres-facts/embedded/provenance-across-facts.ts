import { query } from "@data-stores/psql";

// Reused marker spellings remain ordinary identifiers when only source positions prove binds.
query(sql`SELECT o.id
FROM orders AS o
WHERE o.id = sql_placeholder_1 AND o.tenant_id = ${tenantId}
LIMIT sql_placeholder_2`);

query(sql`SELECT o.id
FROM orders AS o
WHERE o.id = "sql_placeholder_1"
LIMIT "sql_placeholder_1"`);

query(sql`INSERT INTO settings (id, label, status)
VALUES (${id}, sql_placeholder_2, ${status})
ON CONFLICT (id) DO UPDATE
SET label = sql_placeholder_2, status = ${status}`);

query(sql`SELECT 1
WHERE EXISTS (
  SELECT id FROM orders WHERE id = sql_placeholder_1
  UNION ALL
  SELECT id FROM archived_orders WHERE id = sql_placeholder_2
)`);

query(sql`SELECT 1
WHERE EXISTS (
  SELECT id FROM orders WHERE id = ${firstId}
  UNION ALL
  SELECT id FROM archived_orders WHERE id = ${secondId}
)`);

query(sql`SELECT id FROM orders LIMIT ${pageSize}`);

query(sql`UPDATE orders SET name = ${name}
WHERE EXISTS (
  SELECT id FROM orders WHERE id = sql_placeholder_3
  UNION ALL
  SELECT id FROM archived_orders WHERE id = sql_placeholder_4
)`);

query(sql`INSERT INTO settings (id, label)
VALUES (${id}, ${label})
ON CONFLICT (id) DO UPDATE
SET label = EXCLUDED.label
WHERE sql_placeholder_5 IS DISTINCT FROM EXCLUDED.label`);
