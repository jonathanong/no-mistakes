import { query, type QueryExecutor } from "@example/db";

const globalThis: any = {};
globalThis.Map = class {};
const collection = new Map<QueryExecutor, unknown>();
collection.get(query);
