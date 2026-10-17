import { primary } from '@vendor/valkey';
function replacement() {}

function objectAfterWrite(options) {
  const alias = options;
  primary.get = replacement;
  alias.client.get('key');
}
objectAfterWrite({ client: primary });
