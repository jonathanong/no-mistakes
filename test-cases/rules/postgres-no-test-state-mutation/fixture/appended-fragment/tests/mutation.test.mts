import sql from "sql-template-strings";
import { write } from "db";

function sleepSql() {
  return sql`pg_sleep(1)`;
}

export async function exerciseMutation() {
  await write(sql`SELECT `.append(sleepSql()));
}
