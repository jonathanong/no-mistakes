declare function read(...args: any[]): Promise<any>;
declare function factory(): () => string[];
declare function openGroups(): Record<string, string>;
export async function controlNestedFactory() {
  for (const kind of factory()()) await read(kind);
}
export async function controlObjectCall() {
  for (const kind of Object.values(openGroups())) await read(kind);
}
export async function controlArrayAssertion() {
  const kinds = ['post', 'comment'] as Array<string>;
  for (const kind of kinds) await read(kind);
}
export async function controlReadonlyAssertion() {
  const kinds = ['post', 'comment'] as readonly string[];
  for (const kind of kinds) await read(kind);
}
export async function controlScalarAssertion() {
  const kind = 'post' as const;
  for (const letter of kind) await read(letter);
}
export async function controlComputedMethod() {
  const kinds = ['post', 'comment'] as const;
  await Promise.all(kinds['map'](kind => read(kind)));
}
