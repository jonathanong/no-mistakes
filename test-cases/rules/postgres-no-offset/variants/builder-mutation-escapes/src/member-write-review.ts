import sql from "sql-template-strings";
import { read } from "@example/db";

// Writes to builder storage may change SQL without a modeled append call.
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  q.strings = ["SELECT id FROM accounts OFFSET 5"];
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  q["strings"] = ["SELECT id FROM accounts OFFSET 5"];
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  (q as any).strings = ["SELECT id FROM accounts OFFSET 5"];
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  (q.strings as string[]) = ["SELECT id FROM accounts OFFSET 5"];
  read(q); // expect: dynamic
}
// The text projection is safe to read, but assigning it still mutates q.
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  q.text = "SELECT id FROM accounts OFFSET 5";
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  q["text"] = "SELECT id FROM accounts OFFSET 5";
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  (q as any).text = "SELECT id FROM accounts OFFSET 5";
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  (q.text as string) = "SELECT id FROM accounts OFFSET 5";
  read(q); // expect: dynamic
}

// Deletion also mutates builder storage; it is not a read of projected text.
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  delete q.text;
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  delete q["text"];
  read(q); // expect: dynamic
}

// Unverified metadata writes are opaque too; arbitrary setters are not modeled.
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  q.extra = "metadata";
  read(q); // expect: dynamic
}
{
  const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  q["extra"] = "metadata";
  read(q); // expect: dynamic
}

// Replacing a name creates an independent tracked value rather than an escape.
{
  const original = sql`SELECT id FROM accounts WHERE active IS TRUE OFFSET 5`; // expect: offset
  let q = original;
  q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
  read(original);
}
