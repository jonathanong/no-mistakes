import type { SQLStatement } from "sql-template-strings";
import { sql, query } from "@example/db";
const gate = runtimeFlag;

/* The old object is already captured; the final assignment replaces a name. */
let fragment = gate
  ? sql`id FROM users WHERE id = ${firstOwner} OFFSET ${skip}`
  : sql`id FROM users WHERE id = ${secondOwner}`;
const replacement = sql`id FROM replacements OFFSET 9`;
query(sql`SELECT ${fragment} LIMIT ${(fragment = replacement, limit)}`);
query(sql`SELECT ${fragment}`);

/* TypeScript target wrappers still assign only the identifier. */
let asTarget = gate ? sql`1` : sql`2`;
query(sql`SELECT ${asTarget} LIMIT ${((asTarget as SQLStatement) = replacement, 0)}`);
let satisfiesTarget = gate ? sql`3` : sql`4`;
query(sql`SELECT ${satisfiesTarget} LIMIT ${((satisfiesTarget satisfies SQLStatement) = replacement, 0)}`);
let nonNullTarget = gate ? sql`5` : sql`6`;
query(sql`SELECT ${nonNullTarget} LIMIT ${(nonNullTarget! = replacement, 0)}`);
let assertionTarget = gate ? sql`7` : sql`8`;
query(sql`SELECT ${assertionTarget} LIMIT ${((<SQLStatement>assertionTarget) = replacement, 0)}`);

/* Replacing an alias does not mutate the original captured object. */
const original = gate ? sql`id FROM aliases OFFSET 2` : sql`id FROM aliases`;
let alias = original;
query(sql`SELECT ${original} LIMIT ${(alias = replacement, 0)}`);
query(sql`SELECT ${original}`);
let nested = gate ? sql`9` : sql`10`;
let other = replacement;
query(sql`SELECT ${nested} LIMIT ${(nested = (other = replacement), 0)}`);

/* The RHS still runs and may mutate or escape the old object. */
let appended = gate ? sql`id FROM appended` : sql`id FROM appended WHERE active`;
query(sql`SELECT ${appended} LIMIT ${(appended = (appended.append(sql` OFFSET 3`), replacement), 0)}`);
function addOffset(q: SQLStatement): SQLStatement {
  q.append(sql` OFFSET 4`);
  return q;
}
let summarized = gate ? sql`id FROM summaries` : sql`id FROM summaries WHERE active`;
query(sql`SELECT ${summarized} LIMIT ${(summarized = (addOffset(summarized), replacement), 0)}`);
let escaped = gate ? sql`id FROM escapes` : sql`id FROM escapes WHERE active`;
query(sql`SELECT ${escaped} LIMIT ${(escaped = (unknownHelper(escaped), replacement), 0)}`);
let member = gate ? sql`id FROM members` : sql`id FROM members WHERE active`;
const holder = { current: member };
query(sql`SELECT ${member} LIMIT ${(holder.current = member.append(sql` OFFSET 5`), 0)}`);
let wrappedMember = gate ? sql`id FROM wrapped_members` : sql`id FROM wrapped_members WHERE active`;
query(sql`SELECT ${wrappedMember} LIMIT ${((holder.current as SQLStatement) = wrappedMember.append(sql` OFFSET 6`), 0)}`);
let compound = gate ? sql`id FROM compounds` : sql`id FROM compounds WHERE active`;
query(sql`SELECT ${compound} LIMIT ${(compound += sql` OFFSET 7`, 0)}`);

/* Earlier assignment effects retain the existing conservative boundary. */
let earlier = gate ? sql`id FROM earlier` : sql`id FROM earlier WHERE active`;
query(sql`SELECT ${(earlier = replacement, 0)}, ${earlier}`);
query(sql`SELECT ${earlier}`);
