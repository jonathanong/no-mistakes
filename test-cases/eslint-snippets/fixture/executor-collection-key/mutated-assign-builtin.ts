import { query, type QueryExecutor } from "@example/db";

Object.assign = (target) => target;
const map = new Map<QueryExecutor, unknown>();
Object.assign(map, { get: () => undefined });
map.get(query);
