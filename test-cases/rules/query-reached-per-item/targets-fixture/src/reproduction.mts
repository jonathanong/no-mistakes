import { Writable } from 'node:stream';
import { ping } from '@data-stores/psql';
const items = [1, 2];
// Identical spellings are unrelated to the configured database exports.
for (const item of items) process.stdout.write(item);
for (const item of items) process.stderr.write(item);
for (const item of items) console.write(item);
for (const item of items) ping(item);
export async function consume(reader: { read(): Promise<unknown> }) {
  while (true) {
    const item = await reader.read();
    if (!item) break;
  }
}
export async function localParameter(read: (item: number) => Promise<unknown>) {
  await Promise.all(items.map(item => read(item)));
}
export class Writer extends Writable {
  repeat() {
    for (const item of items) this.write(item);
  }
}
