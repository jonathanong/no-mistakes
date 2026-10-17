export function tierSql(): string {
  return "EXISTS (SELECT 1 FROM view_active_plans t WHERE t.tier = 'a')";
}
