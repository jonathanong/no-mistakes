declare function read(...args: any[]): Promise<any>;
type Kind = 'post' | 'comment';
declare function makeGroups(): Record<Kind, any>;
export async function controlTaggedRecordReceiver(items: Record<string, string[]>) {
  const grouped: Record<Kind, any> = makeGroups();
  grouped.post`${items}`;
  await Promise.all(Object.entries(grouped).map(([kind, ids]) => read(kind, ids)));
}
