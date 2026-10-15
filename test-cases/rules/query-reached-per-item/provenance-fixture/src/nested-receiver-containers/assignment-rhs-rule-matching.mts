import { primary as client } from '@vendor/valkey';
function replacement() {}
const box = { wrapper: {} };
const retained = { client };
box.wrapper = retained;
box.wrapper.client.get = replacement;
function consume(value) { value.get('key'); }
for (const item of ['a', 'b']) consume(client);
