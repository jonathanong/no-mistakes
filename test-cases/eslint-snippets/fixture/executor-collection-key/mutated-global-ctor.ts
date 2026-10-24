import { query, type QueryExecutor } from "@example/db";

declare const replacement: any;
globalThis.Map = replacement;
const values = new Map<QueryExecutor, unknown>();
values.get(query);
