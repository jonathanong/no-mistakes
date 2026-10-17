import { query, sql } from "@example/db";
// && returns the null left value, so ?? must take the fallback on that path.
query(((flag ? null : sql`SELECT 1`) && sql`SELECT 2`) ?? sql`SELECT 3`);

const gate = runtimeFlag;
const optional = gate ? null : sql`SELECT 4`;
query((optional && sql`SELECT 5`) ?? sql`SELECT 6`);
query(sql`SELECT 7 ${((gate ? null : sql`unused`) && sql`LIMIT 1`) ?? sql`LIMIT 2`}`);

// false remains non-nullish, and a false executor argument is not SQL.
query(((flag ? false : sql`SELECT 8`) && sql`SELECT 9`) ?? sql`SELECT 10`);
query(((flag ? false : sql`SELECT 11`) && sql`SELECT 12`) || sql`SELECT 13`);
query((null && sql`SELECT 14`) ?? sql`SELECT 15`);
query((false && sql`SELECT 16`) ?? sql`SELECT 17`);

// Empty string is falsy but not nullish; its fragment absence remains distinct.
query(((flag ? "" : sql`SELECT 18`) && sql`SELECT 19`) || sql`SELECT 20`);
query(((flag ? "" : sql`SELECT 21`) && sql`SELECT 22`) ?? sql`SELECT 23`);
// An unknown guard may return a non-nullish scalar and cannot become two SQL arms.
query((unknownGuard && sql`SELECT 24`) ?? sql`SELECT 25`);
