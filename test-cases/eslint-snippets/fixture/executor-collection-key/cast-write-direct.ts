import { query, type QueryExecutor } from "@example/db";

const collection = new Map<QueryExecutor, unknown>();
(collection as any).get = (exec: QueryExecutor) => exec("hidden");
collection.get(query);
