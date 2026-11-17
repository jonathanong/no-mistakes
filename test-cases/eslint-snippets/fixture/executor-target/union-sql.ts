import { read, type QueryExecutor } from "@example/db";

function unionExecutor(query: QueryExecutor | typeof read) {
  query("BEGIN");
}
function typeofExecutor(query: typeof read) {
  query("COMMIT");
}
function mixedExecutor(query: QueryExecutor | ((statement: unknown) => unknown)) {
  query("ROLLBACK");
}

unionExecutor(read);
typeofExecutor(read);
mixedExecutor(read);
