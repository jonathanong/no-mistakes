import { query, type QueryExecutor } from "@example/db";

const assign = Object.assign.bind(Object);
const map = new Map<QueryExecutor, unknown>();
assign(map, { get: () => undefined });
map.get(query);
