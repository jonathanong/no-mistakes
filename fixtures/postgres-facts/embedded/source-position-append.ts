import { query } from "@data-stores/psql";
const statement = sql`SELECT id FROM orders`;

// The appended clause belongs to this later physical line.
statement.append(sql` OFFSET 1`);
query(statement);
const prefix = sql`SELECT id FROM orders\n`;
const suffix = sql`OFFSET 1`;
prefix.append(suffix);
query(prefix);
const suppressed = sql`SELECT id FROM orders`;
suppressed.append(sql` OFFSET 1`); // no-mistakes-disable-line postgres-no-offset
query(suppressed);
const interpolated = sql`SELECT ${1}, ${2}, ${3}, ${4}, ${5}, ${6}, ${7}, ${8}, ${9} FROM orders WHERE id = `;
const tail = sql` \n${
  10
} \
OFFSET 1\n; SELECT id FROM orders OFFSET 0`;
interpolated.append(tail);
query(interpolated);
const same = sql`SELECT id FROM orders`; same.append(sql` OFFSET 0`); query(same);
const notSql = 1;
notSql.append(" OFFSET 1");
const executorDisabled = sql`SELECT id FROM orders`;
executorDisabled.append(sql` OFFSET 1`);
// A legacy executor-call directive must keep suppressing a composed query.
// no-mistakes-disable-next-line postgres-no-offset
query(executorDisabled);
