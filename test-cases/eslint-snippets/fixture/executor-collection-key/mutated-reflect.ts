import { query, type QueryExecutor } from "@example/db";

const map = new Map<QueryExecutor, unknown>();
Reflect.set(map, "delete", () => false);
map.delete(query);
