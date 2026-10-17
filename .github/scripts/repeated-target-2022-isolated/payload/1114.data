import { primary } from '@vendor/valkey';
function replacement() {}

function copiedMethod(client) {
  const get = client.get;
  primary.get = replacement;
  get('key');
}
copiedMethod(primary);
