import { primary } from '@vendor/valkey';
function replacement() {}

function chainedAfterWrite(client) {
  const alias = client;
  const chained = alias;
  primary.get = replacement;
  chained.get('key');
}
chainedAfterWrite(primary);
