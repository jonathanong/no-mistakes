import sql from "sql-template-strings";
import { openTransaction } from "./variants-executor-support";
const gate = runtimeFlag;
const execute = openTransaction();
const statement = sql`SELECT id FROM relative_users`;
if (gate) statement.append(sql` WHERE id = 1`);
execute(statement, statement.append(sql` OFFSET 1`));
