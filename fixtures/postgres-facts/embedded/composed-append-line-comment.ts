import sql from "sql-template-strings";
import { query } from "@data-stores/psql";

declare const clause: string;

const statement = sql`-- list topics
SELECT id FROM topics`;
statement.append(clause);
query(statement);
