import { query } from "@data-stores/psql";

// User-written marker-like identifiers stay columns; only generated positions mark binds.
query(sql`SELECT sql_placeholder_1
FROM orders
WHERE fake_id > sql_placeholder_1 AND id > ${after}
ORDER BY id
LIMIT ${size}`);

query(buildQuery());
