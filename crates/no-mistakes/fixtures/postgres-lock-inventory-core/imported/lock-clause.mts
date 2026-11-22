export function lockClause(): string {
  return " FOR UPDATE";
}

export const helperSql = "SELECT reserveJob(7);\nSELECT 1";

export const firstHelperSql = "SELECT reserveJob(1)";
export const secondHelperSql = "SELECT reserveJob(2)";
