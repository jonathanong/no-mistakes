import { primary } from '@vendor/valkey';
function replacement() {}

function aliasBeforeWrite(client) {
  const alias = client;
  alias.get('key');
  primary.get = replacement;
}
aliasBeforeWrite(primary);
