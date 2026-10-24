import { query, type QueryExecutor } from "@example/db";

(globalThis as any).Map = class {};
const collection = new Map<QueryExecutor, unknown>();
collection.get(query);
