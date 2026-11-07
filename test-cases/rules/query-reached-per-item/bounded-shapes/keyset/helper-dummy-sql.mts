declare function read(...args: any[]): Promise<any>;
async function oneRow(_sql: string, args: unknown[]) {
  return read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 1', args);
}
export async function controlTransitiveSql(initial: string | undefined) {
  let cursor = initial;
  while (cursor) {
    const rows = await oneRow('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
