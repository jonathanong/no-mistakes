import sql from "sql-template-strings";
import { read } from "@bench/db";
const statement = sql`/* mixed */ SELECT 1`;
statement.append(getSuffix());
read(statement);
read(statement.append(getSuffix()));
