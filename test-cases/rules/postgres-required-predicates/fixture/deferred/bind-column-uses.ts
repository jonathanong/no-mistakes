import { query } from "@db";
import SQL from "sql-template-strings";
declare const tenantId: string;
declare const id: string;
declare const sort: number;
// Literal marker-shaped columns coexist with recovered interpolation markers.
query(SQL`SELECT id FROM orders WHERE tenant_id = ${tenantId} AND sql_placeholder_1 = 1 AND "sql_placeholder_1" = 2`);
query(SQL`UPDATE orders SET status = 'x' WHERE tenant_id = ${tenantId} AND sql_placeholder_1 = 1`);
query(SQL`DELETE FROM orders WHERE tenant_id = ${tenantId} AND "sql_placeholder_1" = 1`);
query(SQL`SELECT id FROM orders WHERE sql_placeholder_1 = 1 ORDER BY "sql_placeholder_1"`);
query(SQL`SELECT o.id FROM orders o JOIN tags t ON o.id = ${id} AND t.id = o.sql_placeholder_1 WHERE o.tenant_id = ${tenantId} ORDER BY ${sort}`);
// A generated identifier also cannot become a qualified column or a projection alias reference.
query(SQL`SELECT id FROM orders o WHERE o.${id} = 1 AND o.sql_placeholder_1 = 2`);
query(SQL`SELECT id AS sql_placeholder_1 FROM orders ORDER BY ${sort}`);
query(SQL`SELECT id AS sql_placeholder_1 FROM orders ORDER BY sql_placeholder_1`);
query(SQL`SELECT id AS "sql_placeholder_1" FROM orders ORDER BY "sql_placeholder_1"`);
