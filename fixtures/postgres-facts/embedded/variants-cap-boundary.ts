import { query, sql } from "@example/db";
// Four independent binary choices fill the cap without exceeding it.
query(sql`SELECT 1 ${a ? sql`/* a1 */` : sql`/* a0 */`} ${b ? sql`/* b1 */` : sql`/* b0 */`} ${c ? sql`/* c1 */` : sql`/* c0 */`} ${d ? sql`/* d1 */` : sql`/* d0 */`}`);
