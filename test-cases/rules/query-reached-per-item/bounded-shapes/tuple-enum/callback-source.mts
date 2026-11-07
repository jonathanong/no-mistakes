declare function read(...args: any[]): Promise<any>;
export async function controlCallbackSourceMutation(items: string[]) {
  const kinds = ['post', 'comment'] as const;
  kinds.forEach((_kind, _index, source) => source.push(...items));
  for (const kind of kinds) await read(kind);
}
export async function controlNamedSourceMutation(items: string[]) {
  const kinds = ['post', 'comment'] as const;
  function append(_kind: unknown, _index: number, source: unknown[]) { source.push(...items); }
  kinds.forEach(append);
  for (const kind of kinds) await read(kind);
}
export async function controlArgumentsSourceMutation(items: string[]) {
  const kinds = ['post', 'comment'] as const;
  kinds.forEach(function () { arguments[2].push(...items); });
  for (const kind of kinds) await read(kind);
}
export async function controlRestSourceMutation(items: string[]) {
  const kinds = ['post', 'comment'] as const;
  kinds.forEach((...args) => args[2].push(...items));
  for (const kind of kinds) await read(kind);
}
