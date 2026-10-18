import { primary } from '@vendor/forward-escaped';
function replacement() {}
function write() { alias.client.get = replacement; }
const alias = { client: primary };
publish(write);
function consume(value) { value.get('primary'); }
consume(primary);
