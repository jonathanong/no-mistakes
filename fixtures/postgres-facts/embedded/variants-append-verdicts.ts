import sql from "sql-template-strings";
import { query } from "@example/db";
const c0 = flag0;
const c1 = flag1;
const c2 = flag2;
const c3 = flag3;
const c4 = flag4;
const c5 = flag5;
const q = sql`SELECT 1`;
if (c0) q.append(" /* c0 */");
if (c1) q.append(" /* c1 */");
if (c2) q.append(" /* c2 */");
if (c3) q.append(" /* c3 */");
if (c4) q.append(" /* c4 */");
if (c5) q.append(" /* c5 */");
// This valid OFFSET needs both late choices, whose pair is outside the cap.
if (c4) {
  if (c5) q.append(" OFFSET 1");
}
query(q);
// A concrete OFFSET witness still survives the unexplored alternatives.
q.append(" OFFSET 2");
query(q);
