import { primary } from '@vendor/valkey';
function replacement() {}

(function (client) {
  const alias = client;
  primary.get = replacement;
  alias.get('iife-after-write');
})(primary);
