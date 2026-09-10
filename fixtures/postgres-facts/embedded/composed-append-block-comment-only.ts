import sql from "sql-template-strings";
import { query } from "@data-stores/psql";

declare const statementBody: string;

const statement = sql`/* statement supplied at runtime`;
statement.append(statementBody);
query(statement);
