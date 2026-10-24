import { query, type QueryExecutor } from "@example/db";

const collection = new Map<QueryExecutor, unknown>();
(collection.constructor as any).prototype.get = (exec: QueryExecutor) => exec("hidden");
collection.get(query);
