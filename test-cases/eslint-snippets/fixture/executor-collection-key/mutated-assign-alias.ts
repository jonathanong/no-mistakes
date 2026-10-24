import { query, type QueryExecutor } from "@example/db";

const assign = Object.assign;
const map = new Map<QueryExecutor, unknown>();
assign(map, { get: () => undefined });
map.get(query);
