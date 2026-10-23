import sql from "sql-template-strings";
import { query } from "@example/db";
// Every retained outcome is complete, but the cap leaves an unknown residual.
const capped = sql`INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ON CONFLICT (id) DO NOTHING`;
if (g0) capped.append(" /* cap0 */");
if (g1) capped.append(" /* cap1 */");
if (g2) capped.append(" /* cap2 */");
if (g3) capped.append(" /* cap3 */");
if (g4) capped.append(" /* cap4 */");
if (g5) capped.append(" /* cap5 */");
if (g6) capped.append(" /* cap6 */");
if (g7) capped.append(" /* cap7 */");
if (g8) capped.append(" /* cap8 */");
if (g9) capped.append(" /* cap9 */");
if (g10) capped.append(" /* cap10 */");
if (g11) capped.append(" /* cap11 */");
if (g12) capped.append(" /* cap12 */");
if (g13) capped.append(" /* cap13 */");
if (g14) capped.append(" /* cap14 */");
if (g15) capped.append(" /* cap15 */");
if (g16) capped.append(" /* cap16 */");
if (g17) capped.append(" /* cap17 */");
if (g18) capped.append(" /* cap18 */");
if (g19) capped.append(" /* cap19 */");
query(capped);
// Zero and one loop iterations both demonstrate the missing canonical order.
const repeated = sql`INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ON CONFLICT (id) DO NOTHING`;
while (condition) repeated.append(" /* iteration */");
query(repeated);
// Clear previews cannot prove all future loop iterations clear.
const ordered = sql`INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ORDER BY input.id ON CONFLICT (id) DO NOTHING`;
while (condition) ordered.append(" /* iteration */");
query(ordered);
// A SQL-local exemption on one retained version must not hide another
// version's missing order or the call's unknown residual.
const mixed = gate
  ? sql`INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ON CONFLICT (id) DO NOTHING`
  : sql`/* deadlock-safe */ INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ON CONFLICT (id) DO NOTHING`;
if (m0) mixed.append(" /* mixed0 */");
if (m1) mixed.append(" /* mixed1 */");
if (m2) mixed.append(" /* mixed2 */");
if (m3) mixed.append(" /* mixed3 */");
if (m4) mixed.append(" /* mixed4 */");
if (m5) mixed.append(" /* mixed5 */");
if (m6) mixed.append(" /* mixed6 */");
if (m7) mixed.append(" /* mixed7 */");
if (m8) mixed.append(" /* mixed8 */");
if (m9) mixed.append(" /* mixed9 */");
if (m10) mixed.append(" /* mixed10 */");
if (m11) mixed.append(" /* mixed11 */");
if (m12) mixed.append(" /* mixed12 */");
if (m13) mixed.append(" /* mixed13 */");
if (m14) mixed.append(" /* mixed14 */");
if (m15) mixed.append(" /* mixed15 */");
if (m16) mixed.append(" /* mixed16 */");
if (m17) mixed.append(" /* mixed17 */");
if (m18) mixed.append(" /* mixed18 */");
if (m19) mixed.append(" /* mixed19 */");
query(mixed);
