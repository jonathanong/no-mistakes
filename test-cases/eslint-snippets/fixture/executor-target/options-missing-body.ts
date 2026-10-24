import { read, type QueryExecutor } from "@example/db";
interface Options { query: QueryExecutor }
function load(options: Options) { options.query("SELECT 1"); }
// A parser bridge with no mapped implementation cannot prove this call.
load({ query: read });
