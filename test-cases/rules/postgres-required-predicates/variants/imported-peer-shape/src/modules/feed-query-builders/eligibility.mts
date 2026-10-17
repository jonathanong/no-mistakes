export function buildEligibilityFilter(alias: string, root: string): string {
  return ` AND ${alias}.tier IN ('a','b') AND ${root}.kind = 'root'`;
}
