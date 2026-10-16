import { primary as client } from '@vendor/valkey';
function replacement() {}
const items = ['one'];
const box = { wrapper: { client } };
box.wrapper.client = { get: replacement };
box.wrapper.client.get = replacement;
function replacedTerminal(value) { value.get('key'); }
for (const item of items) { replacedTerminal(client); }
