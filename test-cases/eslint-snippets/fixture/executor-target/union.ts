import { read, write, type QueryExecutor } from "@example/db";
import { unionExecutor, typeofExecutor, mixedExecutor } from "./union-helper";

function unionLocal(query: QueryExecutor | typeof write) {
  query("SELECT 1");
}
function typeofLocal(query: typeof read) {
  query("SELECT 1");
}
function mixedLocal(query: QueryExecutor | ((statement: unknown) => unknown)) {
  query("SELECT 1");
}
function unrelatedLocal(query: QueryExecutor | typeof other) {
  query("SELECT 1");
}

unionLocal(read);
typeofLocal(write);
mixedLocal(read);
unrelatedLocal(read);
unionExecutor(read);
typeofExecutor(write);
mixedExecutor(read);
