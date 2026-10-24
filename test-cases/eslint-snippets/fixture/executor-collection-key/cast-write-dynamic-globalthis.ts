import { query, type QueryExecutor } from "@example/db";

const property: string = "Map";
(globalThis as any)[property] = class {};
const collection = new Map<QueryExecutor, unknown>();
collection.get(query);
