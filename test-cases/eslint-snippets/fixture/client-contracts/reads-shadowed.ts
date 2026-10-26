import { cache as client } from '@app/cache';
async function reads(client) { await client.get('a'); await client.get('b'); }
