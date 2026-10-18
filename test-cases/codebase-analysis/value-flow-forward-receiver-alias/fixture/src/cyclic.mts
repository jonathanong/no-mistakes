import { primary } from '@vendor/forward-cyclic';
function replacement() {}
function write() {
  if (active) write();
  alias.client.get = replacement;
}
const alias = { client: primary };
write();
function consume(value) { value.get('primary'); }
consume(primary);
