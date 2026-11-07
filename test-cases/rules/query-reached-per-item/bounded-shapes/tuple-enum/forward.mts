declare function read(...args: any[]): Promise<any>;
export async function forwardGroups() {
  for (const kind of LATER_KINDS) await read(kind);
  for (const kind of Object.values(LaterKind)) await read(kind);
}
const LATER_KINDS = ['post', 'comment'] as const;
enum LaterKind { Post = 'post', Comment = 'comment' }
export async function controlForwardMutation(items: string[]) {
  async function execute() { for (const kind of kinds) await read(kind); }
  function mutate() { Object.assign(kinds, items); }
  const kinds = ['post', 'comment'] as const;
  mutate();
  await execute();
}
export async function controlForwardEnumMutation(items: string[]) {
  async function execute() { for (const kind of Object.values(LocalKind)) await read(kind); }
  function mutate() { Object.assign(LocalKind, items); }
  enum LocalKind { Post = 'post', Comment = 'comment' }
  mutate();
  await execute();
}
