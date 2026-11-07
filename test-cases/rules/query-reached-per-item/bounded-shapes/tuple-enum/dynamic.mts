declare function read(...args: any[]): Promise<any>;
export async function controlDynamicTuple() {
  const kinds = ['post', 'comment'] as const;
  for (const kind of kinds) {
    eval("kinds.push('post')");
    await read(kind);
  }
}
