// Fixed text: the arguments never reach the SQL, so only the way the call
// site writes them decides whether the call can be reasoned about.
export function gate(first: string, second?: unknown, third?: unknown): string {
  return `EXISTS (SELECT 1 FROM view_active_plans v WHERE v.tier = 'a')`;
}
