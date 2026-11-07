declare function read(...args: any[]): Promise<any>;
export async function controlDynamicRecord(items: Record<string, string[]>) {
  const grouped: Record<'post' | 'comment', string[]> = {} as any;
  eval('Object.assign(grouped, items)');
  await Promise.all(Object.entries(grouped).map(([kind, ids]) => read(kind, ids)));
}
