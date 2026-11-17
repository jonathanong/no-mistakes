import SQL from "sql-template-strings";

export function discoverable(itemColumn: string, eligibility = SQL``) {
  const stmt = SQL`EXISTS (SELECT 1 FROM sources s WHERE s.item_id = `;
  stmt.append(itemColumn);
  stmt.append(SQL` AND s.visible`);
  stmt.append(eligibility);
  stmt.append(SQL`)`);
  return stmt;
}
