import { query, type QueryExecutor } from "@example/db";

const plain = {};
Object.defineProperty(plain, "get", { value: () => undefined });
Object.defineProperty(plain, "unrelated", { value: 1 });
const unrelatedMap = new Map<QueryExecutor, unknown>();
Object.defineProperty(unrelatedMap, "unrelated", { value: 1 });
unrelatedMap.get(query);
const map = new Map<QueryExecutor, unknown>();
map.get(query);
