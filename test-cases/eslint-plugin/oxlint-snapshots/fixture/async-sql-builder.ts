declare function read(statement: string): void;
import { "sql" as tag } from "@example/db";
async function buildQuery() { return tag`SELECT 1`; }
read(buildQuery());
