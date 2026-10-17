import { primary } from '@vendor/valkey';
export function readLatch(client = primary) { return client.get('key'); }
export function diagnostics(client: any) { client.info(); while (true) { client.scan('0'); break; } }
export function wrapper(client: any) { return diagnostics(client); }
