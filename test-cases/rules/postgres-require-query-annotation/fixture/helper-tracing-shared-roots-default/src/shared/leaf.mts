export const leafPrefix = "/* shared_leaf */ ";

export function formatSql(body: string) {
  return leafPrefix + body;
}
