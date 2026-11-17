import { cache as kv } from '@app/cache';

async function sequenceReads() {
  await kv.get('first'), ordinary();
  await kv.get('second');
}
