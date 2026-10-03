import sql from "sql-template-strings";
import { query } from "@example/db";

declare const clause: string;

const statement = sql`-- list topics
SELECT id FROM topics`;
statement.append(clause);
query(statement);
