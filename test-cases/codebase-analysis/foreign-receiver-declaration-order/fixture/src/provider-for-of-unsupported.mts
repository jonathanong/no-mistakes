import { primary, replica, unrelated } from '@vendor/valkey';
let selected = primary;
// Non-declaration and destructured loop heads do not create bare loop bindings.
for (selected of [replica]) { selected.info('assigned'); }
for (const { client } of [{ client: replica }]) { client.info('destructured'); }
for (const client of []) { client.info('empty'); }
for (const client of unrelated) { client.info('non-array'); }
primary.get('primary unchanged');
replica.get('replica unchanged');
