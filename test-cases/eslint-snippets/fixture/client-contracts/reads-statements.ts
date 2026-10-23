import { cache as client } from '@app/cache';
let a, b;
a = await client.get('a');
b = await client.ttl('b');
;
export const exportedA = await client.get('a');
export const exportedB = await client.get('b');
async function returned() {
  await client.get('a');
  return await client.get('b');
  // Unreachable reads must not pair with a return.
  await client.get('unreachable');
}
switch (kind) {
  case 'one':
    await client.get('a');
    await client.get('b');
    break;
  case 'two':
    await client.get('c');
    await client.get('d');
}
;
const first = await client.get('a'), second = await client.get('b'), third = await client.get('c');
await client.get('d');
;
await client.get('e');
const fourth = await client.get('f'), fifth = await client.get('g');
