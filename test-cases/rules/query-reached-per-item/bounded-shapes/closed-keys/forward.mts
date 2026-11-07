declare function read(...args: any[]): Promise<any>;
export async function forwardKeys() {
  const grouped: Record<LaterKind, string[]> = {} as any;
  await Promise.all(Object.entries(grouped).map(([kind, ids]) => read(kind, ids)));
}
type LaterKind = 'post' | 'comment';
