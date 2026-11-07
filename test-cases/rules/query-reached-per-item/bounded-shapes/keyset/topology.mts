declare function read(...args: any[]): Promise<any>;
export async function controlUnbound() { while (missingCursor) { await read('unbound'); } }
export async function controlImmutable() { const cursor = 'start'; while (cursor) { await read('immutable'); } }
export async function controlSingle(cursor: string) { while (cursor) await read('single'); }
export async function controlOpaque(cursor: string) { while (cursor) { eval('cursor = cursor'); await read('opaque'); } }
export async function controlMutableRows(cursor: string) {
  while (cursor) {
    let rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlDestructuredRows(cursor: string) {
  while (cursor) {
    const [rows] = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlRewrittenRows(cursor: string) {
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.at(-1)?.id;
    rows = [];
  }
}
export async function controlUnawaitedRows(cursor: string) {
  while (cursor) {
    const rows = read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlAwaitedValue(cursor: string, supplied: any) {
  while (cursor) {
    const rows = await supplied;
    cursor = rows.at(-1)?.id;
    await read('awaited-value');
  }
}
export async function controlConditionalAdvance(cursor: string) {
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    if (rows.length) cursor = rows.at(-1)?.id;
  }
}
export async function controlSeparatedAdvance(cursor: string) {
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    rows.length;
    cursor = rows.at(-1)?.id;
  }
}
