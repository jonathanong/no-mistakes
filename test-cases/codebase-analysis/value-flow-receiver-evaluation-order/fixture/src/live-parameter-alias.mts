import { primary, replica } from '@vendor/valkey';
function replacement() {}

function liveAfterWrite(client) {
  const alias = client;
  const chained = alias;
  primary.get = replacement;
  chained.get('key');
}
liveAfterWrite(primary);

function aliasBeforeWrite(client) {
  const alias = client;
  alias.get('key');
  primary.get = replacement;
}
aliasBeforeWrite(primary);

function copiedMethod(client) {
  const get = client.get;
  primary.get = replacement;
  get('key');
}
copiedMethod(primary);

function unrelated(client) {
  const alias = client;
  primary.get = replacement;
  alias.set('key', 'value');
}
unrelated(replica);

function objectAlias(options) {
  const alias = options;
  primary.get = replacement;
  alias.client.get('key');
}
objectAlias({ client: primary });
