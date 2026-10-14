import { primary, replica } from '@vendor/valkey';
function replacement() {}

function unrelated(client) {
  const alias = client;
  primary.get = replacement;
  alias.set('key', 'value');
}
unrelated(replica);
