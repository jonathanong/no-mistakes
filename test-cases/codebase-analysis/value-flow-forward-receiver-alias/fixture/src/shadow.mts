import { primary, replica } from '@vendor/forward-shadow';
function replacement() {}
function write() {
  // This later local alias must not inherit the outer receiver identity.
  const alias = { client: replica };
  alias.client.get = replacement;
}
const alias = { client: primary };
write();
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
}
consume(primary, replica);
