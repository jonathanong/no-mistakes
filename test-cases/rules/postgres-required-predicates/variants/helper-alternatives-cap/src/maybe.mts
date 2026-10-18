// An unknown flag leaves two alternatives, and each call site is independent.
export function maybe(flag: boolean): string {
  return flag ? `EXISTS (SELECT 1 FROM view_active_plans v WHERE v.tier = 'a')` : `TRUE`;
}
