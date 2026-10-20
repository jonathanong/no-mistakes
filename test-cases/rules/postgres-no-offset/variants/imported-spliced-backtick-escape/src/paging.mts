export function tail(clause: string): string {
  return `ORDER BY created_at ${clause}`;
}
