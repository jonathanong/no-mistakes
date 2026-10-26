import { cache as client } from '@app/cache';
async function reads() {
  const { key } = await client.get('a');
  const second = await client.get(key);
  if (second) return;
  await client.get('a');
  await other.get('b');
  await client.scan('0');
  await client.get('c');
  await Promise.all([client.get('a'), client.ttl('b')]);
}
function shadow(client) { return client; }
