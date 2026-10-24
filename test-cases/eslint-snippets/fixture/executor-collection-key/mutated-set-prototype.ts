import { query, type QueryExecutor } from "@example/db";

const values = new Map<QueryExecutor, unknown>();
Object.setPrototypeOf(values, { get: (_key: QueryExecutor) => undefined });
values.get(query);
