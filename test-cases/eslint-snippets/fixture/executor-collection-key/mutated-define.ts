import { query, type QueryExecutor } from "@example/db";

const map = new Map<QueryExecutor, unknown>();
Object.defineProperty(map, "get", { value: () => undefined });
map.get(query);
