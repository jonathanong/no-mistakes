import { cache as client } from '@app/cache';
async function reads() {
  const first = await client.get('a');
  const second = await client['ttl']('b');
  await client.get('c');
  await client.ttl('d');
}
