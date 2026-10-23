import { query, sql } from "@example/db";
// Switch paths sharing the same physical seed remain distinct constraints;
// pruning must still favor distinct SQL witnesses over duplicate off paths.
const shared = sql`SELECT 31`;
let q;
switch (mode) {
  case 0: q = shared; break;
  case 1: q = shared; break;
  default: q = sql`SELECT 32`; break;
}
if (g0) q.append(" /* g0 */");
if (g1) q.append(" /* g1 */");
if (g2) q.append(" /* g2 */");
if (g3) q.append(" /* g3 */");
if (g4) q.append(" /* g4 */");
if (g5) q.append(" /* g5 */");
if (g6) q.append(" /* g6 */");
if (g7) q.append(" /* g7 */");
if (g8) q.append(" /* g8 */");
if (g9) q.append(" /* g9 */");
if (g10) q.append(" /* g10 */");
if (g11) q.append(" /* g11 */");
if (g12) q.append(" /* g12 */");
if (g13) q.append(" /* g13 */");
if (g14) q.append(" /* g14 */");
if (g15) q.append(" /* g15 */");
if (g16) q.append(" /* g16 */");
if (g17) q.append(" /* g17 */");
if (g18) q.append(" /* g18 */");
if (g19) q.append(" /* g19 */");
query(q);
