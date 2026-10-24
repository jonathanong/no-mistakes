import { query, type QueryExecutor } from "@example/db";

let collection = new Map<QueryExecutor, unknown>();
for (collection of [new Map<QueryExecutor, unknown>()]) {}
collection.get(query);
