import sql, { type SQLStatement } from "sql-template-strings";
import { write } from "@example/db";

function project(select: SQLStatement): SQLStatement {
  select.append(sql` AND ready = true`);
  return select;
}

declare const mystery: SQLStatement;

write(project(sql`SELECT 1`).append());
write(project(sql`SELECT 1`).append(...[]));
write(project(mystery));
