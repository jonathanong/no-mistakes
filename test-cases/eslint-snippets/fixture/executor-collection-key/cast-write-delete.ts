import { query, type QueryExecutor } from "@example/db";

const collection = new Map<QueryExecutor, unknown>();
delete (collection as any).get;
collection.get(query);
