import sql from "sql-template-strings";
import { query as run } from "@example/db";

export function functionIife(flag: boolean) {
  const built = (function () {
    const query = sql`SELECT p.account_id FROM view_active_plans p`;
    if (!flag) {
      query.append(sql` WHERE p.tier = 'a'`);
    }
    return query;
  })();
  return run(built);
}

export function nestedFunctionAndArrow() {
  const built = (function () {
    const query = sql`SELECT p.account_id FROM view_active_plans p`;
    const unusedFn = function inner() {
      return query;
    };
    const unusedArrow = () => query;
    void unusedFn;
    void unusedArrow;
    return query;
  })();
  return run(built);
}

export function restParameter() {
  const built = (function (..._args: unknown[]) {
    const query = sql`SELECT p.account_id FROM view_active_plans p`;
    return query;
  })();
  return run(built);
}

export function emptyBody() {
  const built = (function () {})();
  return run(built);
}

export function noReturnArgument() {
  const built = (function () {
    const query = sql`SELECT p.account_id FROM view_active_plans p`;
    return;
  })();
  return run(built);
}

export function varLocal() {
  const built = (function () {
    var query = sql`SELECT p.account_id FROM view_active_plans p`;
    return query;
  })();
  return run(built);
}
