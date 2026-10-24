import { query, type QueryExecutor } from "@example/db";

const map = new Map<QueryExecutor, unknown>();
Object.assign(map, { has: () => false });
map.has(query);
