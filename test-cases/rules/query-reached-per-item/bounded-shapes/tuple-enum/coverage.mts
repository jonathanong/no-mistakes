declare function read(...args: any[]): Promise<any>;
export async function controlGuardedTuple(ids: string[]) {
  const kinds = ['post', 'comment'] as const;
  function lookup(id: string) { return read(id); }
  for (const kind of kinds) { await read(kind); ids.map(lookup); }
}
export async function controlAliasInBody(ids: string[]) {
  const kinds = ['post', 'comment'] as const;
  function lookup(id: string) { return read(id); }
  for (const kind of kinds) { const alias = kinds; await read(kind); ids.map(lookup); }
}
