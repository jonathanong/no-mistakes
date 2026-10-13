import { primary } from './clients.mjs';
export function readLocalLatch(client = primary) { return client.get('key'); }
export function localDiagnostics(client: any) { client.info(); while (true) { client.scan('0'); break; } }
