import { write } from '@app/db';
function customTag() { return 'SELECT 1'; }
export function shadowedRawReceiver(String: unknown) {
  String.raw = customTag;
  delete String.raw;
  write(String.raw`/* local receiver */ SELECT 1`); // unanalyzable:raw-property-local-shadow
}
write(String.raw`/* actual builtin */ SELECT 1`); // known:raw-property-global-unchanged
