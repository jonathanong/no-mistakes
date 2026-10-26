import { query } from "@db/b";
import sql from "sql-template-strings";
// Distinct SQL texts keep projection counts independent of parser deduplication.
query("SELECT id FROM scope_b LIMIT 3 OFFSET 2");
export function unused() { return sql`SELECT id FROM fragment_b LIMIT 7`; }
