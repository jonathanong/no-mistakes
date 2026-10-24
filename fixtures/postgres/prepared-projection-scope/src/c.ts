import { query } from "@db/a";
import sql from "sql-template-strings";
// Distinct SQL texts keep projection counts independent of parser deduplication.
query("SELECT id FROM scope_c");
export function unused() { return sql`SELECT id FROM fragment_c`; }
