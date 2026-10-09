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

export function memberTagCanMutateCapturedBuilder() {
  const statement = sql`/* member tag */ SELECT 1`;
  function mutator() { unknownMutation(statement); return ''; }
  const tags = { mutator };
  const ignored = tags.mutator`x`;
  write(statement); // unanalyzable:member-tag-capture
}
