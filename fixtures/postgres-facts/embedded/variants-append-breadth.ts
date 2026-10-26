import sql from "sql-template-strings";
import { query } from "@example/db";
// More decisions than the normal cap: every singleton must survive.
const q = sql`/* bounded */ SELECT 1`;
if (c00) q.append(sql` /* c00 */`);
if (c01) q.append(sql` /* c01 */`);
if (c02) q.append(sql` /* c02 */`);
if (c03) q.append(sql` /* c03 */`);
if (c04) q.append(sql` /* c04 */`);
if (c05) q.append(sql` /* c05 */`);
if (c06) q.append(sql` /* c06 */`);
if (c07) q.append(sql` /* c07 */`);
if (c08) q.append(sql` /* c08 */`);
if (c09) q.append(sql` /* c09 */`);
if (c10) q.append(sql` /* c10 */`);
if (c11) q.append(sql` /* c11 */`);
if (c12) q.append(sql` /* c12 */`);
if (c13) q.append(sql` /* c13 */`);
if (c14) q.append(sql` /* c14 */`);
if (c15) q.append(sql` /* c15 */`);
if (c16) q.append(sql` /* c16 */`);
if (c17) q.append(sql` /* c17 */`);
if (c18) q.append(sql` /* c18 */`);
if (c19) q.append(sql` /* c19 */`);
query(q);

const pairs = sql`SELECT 2`;
if (p0) pairs.append(" /* p0 */");
if (p1) pairs.append(" /* p1 */");
if (p2) pairs.append(" /* p2 */");
if (p3) pairs.append(" /* p3 */");
if (p4) pairs.append(" /* p4 */");
query(pairs);

const invalid = sql`SELECT 3`;
if (broken) invalid.append(" AND");
query(invalid);
