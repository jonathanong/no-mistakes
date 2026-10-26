import { query, type QueryExecutor } from "@example/db";

const collection = new Map<QueryExecutor, unknown>();
const property: string = "get";
(collection as any)[property] = (exec: QueryExecutor) => exec("hidden");
collection.get(query);
