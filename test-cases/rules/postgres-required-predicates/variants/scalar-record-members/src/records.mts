const records = Object.freeze({ one: { table: 'items', column: 'id' } } as const);
export function config(key: string) {
  if (!Object.hasOwn(records, key)) throw new Error('unknown key');
  return records[key as keyof typeof records];
}
export function scalar(status: string, at: Date) {
  return { aAt: status === 'x' ? at : null };
}
