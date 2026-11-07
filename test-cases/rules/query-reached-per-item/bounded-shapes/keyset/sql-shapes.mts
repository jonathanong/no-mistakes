declare function read(...args: any[]): Promise<any>;
export async function positiveQuoted(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT \"id\" FROM \"items\" WHERE \"id\" > $1 ORDER BY \"id\" LIMIT 100;", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function positiveWildcard(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT * FROM items WHERE id > $1 ORDER BY id LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function positiveQualifiedWildcard(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT p.* FROM public.items p WHERE id > $1 ORDER BY id LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function positiveImplicitAlias(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT id id FROM items WHERE id > $1 ORDER BY id LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function positiveExplicitAlias(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT id AS id FROM items WHERE id > $1 ORDER BY id LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlBrokenLiteral(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT 'unterminated", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlMissingWhere(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT id FROM items LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlNonselect(cursor: string) {
  while (cursor) {
    const rows = await read("DELETE FROM items WHERE id > $1 ORDER BY id LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlMissingFrom(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT id WHERE id > $1 ORDER BY id LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlMissingTable(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT id FROM WHERE id > $1 ORDER BY id LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlDeepTable(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT id FROM db.public.items WHERE id > $1 ORDER BY id LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlInvalidTableAlias(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT id FROM items 7 WHERE id > $1 ORDER BY id LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlUnsupportedJoin(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT id FROM items JOIN more WHERE id > $1 ORDER BY id LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlMissingProjection(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT FROM items WHERE id > $1 ORDER BY id LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlWrongQualifier(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT other.id FROM items p WHERE id > $1 ORDER BY id LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlNumericImplicitAlias(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT id 7 FROM items WHERE id > $1 ORDER BY id LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlNumericExplicitAlias(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT id AS 7 FROM items WHERE id > $1 ORDER BY id LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
export async function controlExpressionProjection(cursor: string) {
  while (cursor) {
    const rows = await read("SELECT id + 1 FROM items WHERE id > $1 ORDER BY id LIMIT 100", [cursor]);
    cursor = rows.at(-1)?.id;
  }
}
