import { cache as client } from '@app/cache';
async function reads() {
  const first = (await client.get('a')) as string;
  const second = await client.get(`${first}:next`);
  // Separate scenarios so the next first read is not itself adjacent.
  ;
  const [key] = await client.get('a');
  await client.get(key);
  ;
  const { deep: { nested } } = await client.get('a');
  await client.get(nested);
}
