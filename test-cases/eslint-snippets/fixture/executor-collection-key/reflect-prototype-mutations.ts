import { query, type QueryExecutor } from "@example/db";

Reflect.defineProperty(Map.prototype, "get", { value: (_key: QueryExecutor) => undefined });
const map = new Map<QueryExecutor, unknown>();
map.get(query);

Reflect.setPrototypeOf(Set.prototype, {});
const set = new Set<QueryExecutor>();
set.has(query);

const dynamicName: string = "has";
Reflect.defineProperty(Set.prototype, dynamicName, { value: (_key: QueryExecutor) => false });
set.has(query);
