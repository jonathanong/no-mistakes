import sql from "sql-template-strings";
import { query } from "@example/db";

declare const fragment: { text: string };

const statement = sql`SELECT id FROM topics`;
statement.append(fragment.text);
query(statement);
