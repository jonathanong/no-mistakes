import { primary } from '@vendor/forward-uncalled';
function replacement() {}
function write() { alias.client.get = replacement; }
const alias = { client: primary };
function consume(value) { value.get('primary'); }
consume(primary);
