import { primary } from '@vendor/valkey';
function replacement() {}

function aliasAfterWrite(client) {
  const alias = client;
  primary.get = replacement;
  alias.get('key');
}
aliasAfterWrite(primary);
