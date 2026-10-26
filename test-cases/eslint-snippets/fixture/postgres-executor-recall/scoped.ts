import { makeExecutor, type Executor } from "@example/db";
const execute = makeExecutor();
const alias = execute;
alias("BEGIN");
function run(executor: Executor) {
  const q = executor;
  q("COMMIT");
  function shadow(executor) { executor("ROLLBACK"); }
}
