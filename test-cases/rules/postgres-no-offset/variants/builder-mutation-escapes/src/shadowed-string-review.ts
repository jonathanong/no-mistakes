import sql from "sql-template-strings";
import { read } from "@example/db";

// An escaping inner builder cannot mutate an outer primitive with the same name.
{
  const q = "SELECT id FROM accounts WHERE active IS TRUE OFFSET 5"; // expect: offset
  {
    const q = sql`SELECT id FROM accounts WHERE active IS TRUE`;
    const retained = [q];
  }
  read(q);
}
