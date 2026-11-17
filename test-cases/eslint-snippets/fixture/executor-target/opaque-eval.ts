import { read, type QueryExecutor } from "@example/db";

const opaqueEval = (query: QueryExecutor) => {
  eval("void 0");
  return query("SELECT id FROM t");
};

opaqueEval(read);
