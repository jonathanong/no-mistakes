import type { SQLStatement } from "sql-template-strings";
import { sql, query } from "@example/db";
const gate = runtimeFlag;
const replacement = sql`id FROM replacements OFFSET 9`;
let fragment = gate ? sql`id FROM pages OFFSET 1` : sql`id FROM pages`;
query(sql`SELECT ${fragment} LIMIT ${(fragment = replacement, 0)}`);
query(sql`SELECT ${fragment}`);
let wrapped = gate ? sql`id FROM wrapped_pages OFFSET 2` : sql`id FROM wrapped_pages`;
query(sql`SELECT ${wrapped} LIMIT ${((wrapped as SQLStatement) = replacement, 0)}`);
const original = gate ? sql`id FROM aliases OFFSET 3` : sql`id FROM aliases`;
let alias = original;
query(sql`SELECT ${original} LIMIT ${(alias = replacement, 0)}`);
let safe = gate ? sql`id FROM safe_pages WHERE active` : sql`id FROM safe_pages`;
query(sql`SELECT ${safe} LIMIT ${(safe = replacement, 0)}`);

/* The right-hand side can still alter the previously captured object. */
let appended = gate ? sql`id FROM appended_pages` : sql`id FROM appended_pages WHERE active`;
query(sql`SELECT ${appended} LIMIT ${(appended = (appended.append(sql` OFFSET 6`), replacement), 0)}`);
function addOffset(q: SQLStatement): SQLStatement {
  q.append(sql` OFFSET 7`);
  return q;
}
let summarized = gate ? sql`id FROM summaries` : sql`id FROM summaries WHERE active`;
query(sql`SELECT ${summarized} LIMIT ${(summarized = (addOffset(summarized), replacement), 0)}`);
let escaped = gate ? sql`id FROM escapes` : sql`id FROM escapes WHERE active`;
query(sql`SELECT ${escaped} LIMIT ${(escaped = (unknownHelper(escaped), replacement), 0)}`);

/* Recovered branch and host suppressions retain physical source ownership. */
let suppressedBranch = gate
  // no-mistakes-disable-next-line postgres-no-offset
  ? sql`id FROM suppressed_pages OFFSET 4`
  : sql`id FROM suppressed_pages`;
query(sql`SELECT ${suppressedBranch} LIMIT ${(suppressedBranch = replacement, 0)}`);
let suppressedHost = gate ? sql`id FROM host_pages OFFSET 5` : sql`id FROM host_pages`;
// no-mistakes-disable-next-line postgres-no-offset
query(sql`SELECT ${suppressedHost} LIMIT ${(suppressedHost = replacement, 0)}`);
let suppressedMutation = gate ? sql`id FROM hidden_pages` : sql`id FROM hidden_pages WHERE active`;
// no-mistakes-disable-next-line postgres-no-offset
query(sql`SELECT ${suppressedMutation} LIMIT ${(suppressedMutation = (suppressedMutation.append(sql` OFFSET 8`), replacement), 0)}`);
