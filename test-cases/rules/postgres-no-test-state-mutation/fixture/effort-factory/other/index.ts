export async function beginTransaction(): Promise<(sql: string) => unknown> {
  throw new Error('This same-spelled factory is not configured');
}
