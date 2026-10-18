import sql, { type SQLStatement } from "sql-template-strings";
import { read } from "@example/db";

function supported(q: SQLStatement): SQLStatement {
  q.append(sql` OFFSET 5`); // expect: offset
  return q;
}
const statement = sql`SELECT id FROM accounts WHERE active IS TRUE`;
read(supported(statement));

const independent = sql`SELECT id FROM accounts WHERE active IS TRUE`;
const escaped = sql`SELECT id FROM accounts WHERE active IS TRUE`;
opaque(escaped);
read(independent);

const text = "SELECT id FROM accounts WHERE active IS TRUE";
const copied = text;
opaque(text);
read(runtimeFlag ? copied : "SELECT id FROM accounts WHERE active IS TRUE");

const beforeReplacement = sql`SELECT id FROM accounts WHERE active IS TRUE`;
let replaced = beforeReplacement;
replaced = sql`SELECT id FROM accounts WHERE active IS TRUE`;
opaque(replaced);
read(beforeReplacement);
