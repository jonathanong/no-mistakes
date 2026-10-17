import sql from "sql-template-strings";
import { query as run } from "@example/db";

export function noPlaceholder(flag: boolean) {
  const built = (() => {
    const query = sql`SELECT p.account_id FROM view_active_plans p`;
    if (!flag) {
      query.append(sql` WHERE p.tier = 'a'`);
    }
    return query;
  })();
  return run(built);
}

export function localGuard() {
  const built = (() => {
    const flag = Math.random() > 0.5;
    const query = sql`SELECT p.account_id FROM view_active_plans p`;
    if (!flag) {
      query.append(sql` WHERE p.tier = 'a'`);
    }
    return query;
  })();
  return run(built);
}
