import { primary } from '@vendor/forward-uncalled-after';
function replacement() {}
const alias = { client: primary };
function write() { alias.client.get = replacement; }
function consume(value) { value.get('primary'); }
consume(primary);
