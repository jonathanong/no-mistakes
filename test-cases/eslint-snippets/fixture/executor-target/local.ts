import { read, write, type QueryExecutor } from "@example/db";
function run(exec: QueryExecutor) { exec("SELECT 1"); }
const arrow = (exec?: QueryExecutor) => exec?.("SELECT 1");
const expression = function(exec: QueryExecutor) { exec("SELECT 1"); };
function object({ query }: { query: QueryExecutor }) { query("SELECT 1"); }
run(read);
arrow(read);
expression(read);
object({ query: read });
// A typed target does not make a forwarding callback analyzable.
run((statement) => read(statement));
function defaulted(exec: QueryExecutor = read) { exec("SELECT 1"); }
function renamed({ query: execute }: { query: QueryExecutor }) { execute("SELECT 1"); }
defaulted(read);
renamed({ query: read });
run(flag ? read : write);
run(flag ? ((statement) => read(statement)) : read);
