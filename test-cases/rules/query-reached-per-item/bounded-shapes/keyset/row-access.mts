declare function read(...args: any[]): Promise<any>;
export async function positiveNonoptional(cursor: string, sql = 'SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', saved?: any) {
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.at(-1).id;
  }
}
export async function controlOptionalRowCall(cursor: string, sql = 'SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', saved?: any) {
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows?.at(-1);
  }
}
export async function controlComputedRow(cursor: string, sql = 'SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', saved?: any) {
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows[0];
  }
}
export async function controlResultProperty(cursor: string, sql = 'SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', saved?: any) {
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.id;
  }
}
export async function controlComputedAt(cursor: string, sql = 'SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', saved?: any) {
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows['at'](-1).id;
  }
}
export async function controlOtherMethod(cursor: string, sql = 'SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', saved?: any) {
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.first(-1).id;
  }
}
export async function controlPositiveIndex(cursor: string, sql = 'SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', saved?: any) {
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.at(+1).id;
  }
}
export async function controlMissingParameters(cursor: string, sql = 'SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', saved?: any) {
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100');
    cursor = rows.at(-1)?.id;
  }
}
export async function controlNonarrayParameters(cursor: string, sql = 'SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', saved?: any) {
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', cursor);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlDynamicSql(cursor: string, sql = 'SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', saved?: any) {
  while (cursor) {
    const rows = await read(sql, [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlResultAssignmentEscape(cursor: string, sql = 'SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', saved?: any) {
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.at(-1)?.id;
    saved = rows;
  }
}
