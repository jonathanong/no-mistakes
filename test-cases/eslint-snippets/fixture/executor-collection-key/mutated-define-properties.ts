import { query, type QueryExecutor } from "@example/db";

declare const replacement: unknown;
const values = new Map<QueryExecutor, unknown>();
Object.defineProperties(values, { get: { value: replacement } });
values.get(query);
