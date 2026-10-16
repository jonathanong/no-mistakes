import { read } from './psql.mjs';
import { primary } from '@vendor/valkey';
async function make() { return read; }
export const unawaited = make();
export const awaited = await make();
function configured(client = primary) { return client.get('key'); }
export const known = configured.bind(null, primary);
function other(client = primary) { return client.get('key'); }
export const wrong = other.bind(null, unrelated);
