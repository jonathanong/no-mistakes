import sql, { type SQLStatement } from "sql-template-strings";
import { read } from "@example/db";

// These uses preserve modeled ownership rather than escaping a mutable object.
const receiver = sql`SELECT id FROM accounts WHERE active IS TRUE`;
receiver.append(sql` ORDER BY id`);
read(receiver);
read(receiver);
const copiedText = receiver.text;
opaque(copiedText);
opaque(receiver["text"]);
read(receiver);

const original = sql`SELECT id FROM accounts WHERE active IS TRUE`;
const alias = original;
read(original);

function typed(q: SQLStatement): SQLStatement {
  q.append(sql` ORDER BY id`);
  return q;
}
const typedArgument = sql`SELECT id FROM accounts WHERE active IS TRUE`;
read(typed(typedArgument));

// Type-only references do not expose an object at runtime.
type OriginalStatement = typeof original;
read(original);

// Trusted SQL copy factories consume fragments without retaining their builders.
const factoryInput = sql`SELECT id FROM accounts WHERE active IS TRUE`;
sql(factoryInput);
sql.join([factoryInput]);
read(factoryInput);
