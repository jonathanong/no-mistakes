import type { QueryExecutor } from "@example/db";

export function foreignTyped(query: QueryExecutor) {
  query("SELECT 1 OFFSET 103"); // EXPECT_OFFSET FOREIGN_CLAUSE EXPECT_KNOWN_CALL
}
