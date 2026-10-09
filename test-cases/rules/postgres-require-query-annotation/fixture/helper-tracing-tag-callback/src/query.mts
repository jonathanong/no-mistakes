import sql from 'sql-template-strings';
import opaqueTag from 'opaque-tag';
import { write } from '@app/db';

export function untrustedTagCanInvokeItsInterpolationCallback() {
  const statement = sql`/* before interpolation callback */ SELECT 1`;
  const callback = () => unknownMutation(statement);
  // An unknown tag may invoke an interpolated callback before this sink.
  const ignored = opaqueTag`${callback}`;
  write(statement); // unanalyzable:untrusted-tag-callback-capture
}
