import { cache as client } from '@app/cache';
async function reads() {
  // Each transparent receiver wrapper preserves the same imported client binding.
  await (client as Cache).get('a');
  await client!.get('b');
  await (client satisfies Cache).get('c');
  await (client as Cache)['ttl']('d');
}
