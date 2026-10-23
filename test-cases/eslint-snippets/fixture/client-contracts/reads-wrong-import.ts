import type { cache as client } from '@app/cache';
async function reads() { await client.get('a'); await client.get('b'); }
