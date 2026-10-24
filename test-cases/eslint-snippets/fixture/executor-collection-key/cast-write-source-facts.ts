import { query, type QueryExecutor } from "@example/db";

let counter = 0;
++counter;
counter++;
for (const value of []) void value;

// Direct eval makes source-wide collection facts uncertain.
(eval)("void 0");
const collection = new Map<QueryExecutor, unknown>();
collection.get(query);
