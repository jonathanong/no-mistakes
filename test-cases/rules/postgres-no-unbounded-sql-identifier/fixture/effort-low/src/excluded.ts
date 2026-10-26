import { query } from "@example/db";
declare function sqlHelper(): string;
query(sqlHelper()); // excluded: rule-local scope
