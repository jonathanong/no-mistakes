export function lockClause(): string {
  return " FOR UPDATE";
}

export const helperSql = "SELECT reserveJob(7)";
