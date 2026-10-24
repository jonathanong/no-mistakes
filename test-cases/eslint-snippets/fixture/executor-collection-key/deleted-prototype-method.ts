import { query, type QueryExecutor } from "@example/db";

delete (Map.prototype as any).get;
const values = new Map<QueryExecutor, unknown>();
values.get(query);
