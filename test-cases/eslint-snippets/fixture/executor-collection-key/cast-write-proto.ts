import { query, type QueryExecutor } from "@example/db";

const collection = new Map<QueryExecutor, unknown>();
(collection as any).__proto__ = {};
collection.get(query);
