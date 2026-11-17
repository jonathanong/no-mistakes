import { read, type QueryExecutor } from "@example/db";

const wrapped = (query: QueryExecutor) => {
  const fragment = "SELECT hidden";
  (eval as typeof eval)("query(fragment)");
  return query("SELECT id FROM t");
};

wrapped(read);
