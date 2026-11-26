export const leafPrefix = "/* query: leaf */ ";

export function formatLeafSql(table: string) {
  return leafPrefix + "SELECT id FROM " + table;
}
