import { query, type QueryExecutor } from "@example/db";

let leaked: any;
const cache = new Map<QueryExecutor, unknown>();
leaked = cache;
leaked.get = (exec: QueryExecutor) => exec("hidden");
cache.get(query);
