declare function read(...args: any[]): Promise<any>;
export async function bounded(initial: string | undefined) {
  let cursor = initial;
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function control(ids: string[]) {
  for (const id of ids) await read(id);
}
export async function controlNoLimit(initial: string | undefined) {
  let cursor = initial;
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id', [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlWrongKey(initial: string | undefined) {
  let cursor = initial;
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.at(-1)?.other;
  }
}
export async function controlMixed(initial: string | undefined) {
  let cursor = initial;
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.at(-1)?.id;
    await read('extra');
  }
}
export async function controlOne(initial: string | undefined) {
  let cursor = initial;
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 1', [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function qualifiedPage(initial: string | undefined) {
  let cursor = initial;
  while (cursor) {
    const rows = await read(`SELECT p.id, p.name FROM public.items AS p WHERE id > $1 ORDER BY id ASC LIMIT 100`, [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlProjection(initial: string | undefined) {
  let cursor = initial;
  while (cursor) {
    const rows = await read('SELECT name AS id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlOffset(initial: string | undefined) {
  let cursor = initial;
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100 OFFSET 1', [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlParameter(initial: string | undefined, other: string) {
  let cursor = initial;
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [other]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlFirst(initial: string | undefined) {
  let cursor = initial;
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.at(0)?.id;
  }
}
export async function controlEscape(initial: string | undefined) {
  let cursor = initial;
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.at(-1)?.id;
    consume(rows);
  }
}
declare function consume(rows: unknown): void;
export async function controlOuter(starts: string[]) {
  for (const initial of starts) {
    let cursor = initial;
    while (cursor) {
      const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
      cursor = rows.at(-1)?.id;
    }
  }
}
export async function parameterPage(cursor: string | undefined) {
  while (cursor) {
    const rows = await read('SELECT id FROM items WHERE id > $1 ORDER BY id LIMIT 100', [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
