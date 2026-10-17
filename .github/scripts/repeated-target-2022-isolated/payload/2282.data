import { primary } from '@vendor/valkey';
function replacement() {}

hoisted(primary);
function hoisted(client) {
  const alias = client;
  primary.get = replacement;
  alias.get('hoisted-after-write');
}
