declare function read(...args: any[]): Promise<any>;
const KINDS = ['post', 'comment'] as const;
enum Kind { Post = 'post', Comment = 'comment' }
export async function bounded() {
  for (const kind of KINDS) await read(kind);
  for (const kind of Object.values(Kind)) await read(kind);
}
export async function control(kinds: string[]) {
  for (const kind of kinds) await read(kind);
}
export async function inlineTuple() {
  for (const kind of ['post', 'comment'] as const) await read(kind);
}
export async function controlArray() {
  const kinds = ['post', 'comment'];
  for (const kind of kinds) await read(kind);
}
export async function controlShadow(KINDS: string[]) {
  for (const kind of KINDS) await read(kind);
}
export async function controlMutation() {
  const kinds = ['post', 'comment'] as const;
  kinds.push('post');
  for (const kind of kinds) await read(kind);
}
export async function controlAlias() {
  const kinds = ['post', 'comment'] as const;
  const alias = kinds;
  for (const kind of kinds) await read(kind);
}
export async function controlBodyMutation() {
  const kinds = ['post', 'comment'] as const;
  for (const kind of kinds) { kinds.push('post'); await read(kind); }
}
export async function fixedTuple(first: string, second: string) {
  const kinds = [first, second] as const;
  for (const kind of kinds) await read(kind);
}
export async function controlSpread(kinds: string[]) {
  const values = ['post', ...kinds] as const;
  for (const kind of values) await read(kind);
}
export async function controlReturnedAlias() {
  const kinds = ['post', 'comment'] as const;
  function expose() { return kinds; }
  expose().push('post');
  for (const kind of kinds) await read(kind);
}
export async function controlParameterAlias() {
  const kinds = ['post', 'comment'] as const;
  function mutate(values = kinds) { values.push('post'); }
  mutate();
  for (const kind of kinds) await read(kind);
}
export async function controlLateTuple(items: string[]) {
  const kinds = ['post', 'comment'] as const;
  async function execute() { for (const kind of kinds) await read(kind); }
  Object.assign(kinds, items);
  await execute();
}
export async function controlLateEnum(items: string[]) {
  enum LocalKind { Post = 'post', Comment = 'comment' }
  async function execute() { for (const kind of Object.values(LocalKind)) await read(kind); }
  Object.assign(LocalKind, items);
  await execute();
}
