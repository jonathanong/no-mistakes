export function statement(): string {
  return `SELECT id FROM accounts
-- no-mistakes-disable-next-line postgres-no-offset
WHERE id = 1`;
}
