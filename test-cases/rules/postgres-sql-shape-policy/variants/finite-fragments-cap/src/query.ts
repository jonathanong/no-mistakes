import { query, sql } from "@example/db";
const statement = sql`SELECT id FROM accounts`;
statement.append((flag0 ? (flag1 ? (flag2 ? (flag3 ? sql` WHERE id = 0` : sql` WHERE id = 1`) : (flag4 ? sql` WHERE id = 2` : sql` WHERE id = 3`)) : (flag5 ? (flag6 ? sql` WHERE id = 4` : sql` WHERE id = 5`) : (flag7 ? sql` WHERE id = 6` : sql` WHERE id = 7`))) : (flag8 ? (flag9 ? (flag10 ? sql` WHERE id = 8` : sql` WHERE id = 9`) : (flag11 ? sql` WHERE id = 10` : sql` WHERE id = 11`)) : (flag12 ? (flag13 ? sql` WHERE id = 12` : sql` WHERE id = 13`) : (flag14 ? sql` WHERE id = 14` : (flag15 ? sql` WHERE id = 15` : sql` WHERE id = 16`))))));
query(statement);
// Balanced branches exercise the version cap before the depth limit.
