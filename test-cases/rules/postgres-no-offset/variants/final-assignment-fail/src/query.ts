import type { SQLStatement } from "sql-template-strings";
import { sql, query } from "@example/db";
const gate = runtimeFlag;
let other = 0;
const direct = gate ? sql`id FROM pages` : sql`id FROM pages WHERE active`;
query(sql`SELECT ${direct} LIMIT ${((other = direct).append(sql` OFFSET 1`), 0)}`);
const computed = sql`id FROM computed_pages`;
query(sql`SELECT ${computed} LIMIT ${((other = computed)["append"](sql` OFFSET 2`), 0)}`);
function addOffset(q: SQLStatement): SQLStatement {
  q.append(sql` OFFSET 3`);
  return q;
}
const summarized = gate ? sql`id FROM summaries` : sql`id FROM summaries WHERE active`;
query(sql`SELECT ${summarized} LIMIT ${(addOffset(other = summarized), 0)}`);
const wrapped = gate ? sql`id FROM wrapped_pages` : sql`id FROM wrapped_pages WHERE active`;
query(sql`SELECT ${wrapped} LIMIT ${(((other as unknown) = wrapped).append(sql` OFFSET 4`), 0)}`);
const nested = gate ? sql`id FROM nested_pages` : sql`id FROM nested_pages WHERE active`;
let nestedAlias = 0;
query(sql`SELECT ${nested} LIMIT ${((other = (nestedAlias = nested))["append"](sql` OFFSET 5`), 0)}`);
const escaped = gate ? sql`id FROM escaped_pages` : sql`id FROM escaped_pages WHERE active`;
query(sql`SELECT ${escaped} LIMIT ${(unknownHelper(other = escaped), 0)}`);

/* A discarded assignment result still preserves the captured OFFSET. */
const unchanged = gate ? sql`id FROM unchanged_pages OFFSET ${skip}` : sql`id FROM unchanged_pages`;
query(sql`SELECT ${unchanged} LIMIT ${(other = unchanged, limit)}`);
query(sql`SELECT ${other}`);
const suppressedBranch = gate
  // no-mistakes-disable-next-line postgres-no-offset
  ? sql`id FROM suppressed_pages OFFSET 9`
  : sql`id FROM suppressed_pages`;
query(sql`SELECT ${suppressedBranch} LIMIT ${(other = suppressedBranch, 0)}`);
const suppressedMutation = gate ? sql`id FROM hidden_pages` : sql`id FROM hidden_pages WHERE active`;
// no-mistakes-disable-next-line postgres-no-offset
query(sql`SELECT ${suppressedMutation} LIMIT ${((other = suppressedMutation).append(sql` OFFSET 10`), 0)}`);
