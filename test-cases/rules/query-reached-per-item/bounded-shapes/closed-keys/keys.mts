declare function read(...args: any[]): Promise<any>;
export async function closedKeys() {
  const grouped: Record<'post' | 'comment', string[]> = {} as any;
  for (const kind in grouped) {
    const batch = grouped[kind];
    await read(kind, batch);
  }
}
export async function controlOpenKeys(grouped: Record<string, string[]>) {
  for (const kind in grouped) {
    const batch = grouped[kind];
    await read(kind, batch);
  }
}
