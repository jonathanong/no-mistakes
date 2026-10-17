import { primary as client } from '@vendor/valkey';
function replacement() {}
const items = ['one'];
const box = { wrapper: { client } };
const savedWrapper = box.wrapper;
box.wrapper = {};
savedWrapper.client.get = replacement;
function acquiredBeforeContainer(value) { value.get('key'); }
for (const item of items) { acquiredBeforeContainer(client); }
