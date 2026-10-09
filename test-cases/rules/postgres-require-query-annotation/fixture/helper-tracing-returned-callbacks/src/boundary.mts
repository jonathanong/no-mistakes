import sql from "sql-template-strings";
import { write } from "@app/db";

const statement = sql`/* callback boundary */ SELECT 1`;
function callback() {
  write(statement);
}
