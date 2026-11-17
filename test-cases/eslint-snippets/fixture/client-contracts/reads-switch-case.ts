import { cache as kv } from '@app/cache';

async function switchReads(kind: string) {
  switch (kind) {
    case 'read':
      Number(await kv.ttl('first'));
      Boolean(await kv.exists('second'));
      break;
    case 'other':
      await kv.get('other');
  }
}
