import { query } from "@db/b";
import sql from "sql-template-strings";
// Distinct SQL texts keep projection counts independent of parser deduplication.
query("SELECT id FROM scope_d");
export function unused() { return sql`SELECT id FROM fragment_d`; }
