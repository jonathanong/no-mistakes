export function branching(): string {
  if (process.env.STRICT) {
    return "EXISTS (SELECT 1 FROM view_active_plans p WHERE p.account_id = 1)";
  }
  return "TRUE";
}

export function looping(): string {
  let sql = "TRUE";
  for (const tier of ["a", "b"]) {
    sql += ` AND EXISTS (SELECT 1 FROM view_active_plans p WHERE p.tier = '${tier}')`;
  }
  return sql;
}

export function throwing(): string {
  if (!process.env.ALLOWED) throw new Error("not allowed");
  return "EXISTS (SELECT 1 FROM view_active_plans p WHERE p.account_id = 1)";
}
