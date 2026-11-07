import { read } from '@example/db';
export async function local(read: (sql: string) => Promise<void>) {
  await read('SELECT 1');
}
