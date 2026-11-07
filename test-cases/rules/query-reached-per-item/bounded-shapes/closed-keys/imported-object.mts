import { Object } from './fake.mjs';
declare function read(...args: any[]): Promise<any>;
export async function controlImportedObject() {
  const grouped: Record<'post' | 'image', string[]> = {} as any;
  await Promise.all(Object.entries(grouped).map(([kind, ids]) => read(kind, ids)));
}
