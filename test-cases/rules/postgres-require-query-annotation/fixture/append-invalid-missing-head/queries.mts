import sql from "sql-template-strings";
import { read } from "@bench/db";
// Complete invalid SQL still has a decidable missing annotation head.
read(sql`SELECT (`.append(" AND"));
const conditional = sql`SELECT (`;
if (flag) conditional.append(" AND");
else conditional.append(" OR");
read(conditional);
read(sql`/* */ SELECT (`.append(" AND"));
read(sql`BEGINNING (`.append(" AND"));
// Named heads and transaction exemptions are independent negative controls.
read(sql`/* named */ SELECT (`.append(" AND"));
read(sql`BEGIN (`.append(" AND"));
// no-mistakes-disable-next-line postgres-require-query-annotation
read(sql`SELECT (`.append(" AND"));
read(sql`SELECT (`.append(" AND")); // no-mistakes-disable-line postgres-require-query-annotation
