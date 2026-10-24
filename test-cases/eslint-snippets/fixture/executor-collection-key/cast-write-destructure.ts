import { query, type QueryExecutor } from "@example/db";

const collection = new Map<QueryExecutor, unknown>();
({ get: (collection as any).get } = { get: (exec: QueryExecutor) => exec("hidden") });
collection.get(query);
