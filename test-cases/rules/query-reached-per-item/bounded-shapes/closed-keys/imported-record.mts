declare function read(...args: any[]): Promise<any>;
export async function controlImportedRecord() {
  const grouped: Record<'post' | 'image', string[]> = {} as any;
  await Promise.all(Object.entries(grouped).map(([kind, ids]) => read(kind, ids)));
}
import type { Record } from './fake.mjs';
