import { primary } from '@vendor/valkey';
//----------------------------------------------------------------------------
export function make() {
  const opts = { client: primary };
  primary.info('key');
  primary.get = replacement;
  return opts;
}
function replacement() {}
