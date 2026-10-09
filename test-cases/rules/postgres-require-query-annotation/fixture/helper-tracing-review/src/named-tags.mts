import { write } from '@app/db';
import { sql, default as tag, sql as unconfiguredAlias } from 'sql-template-strings';
export function namedTags(id: string) {
  write(sql`/* named import */ SELECT ${id}`); // known:named-import
  write(tag`/* default as */ SELECT ${id}`); // known:default-as-import
  write(unconfiguredAlias`/* arbitrary alias */ SELECT ${id}`); // unanalyzable:unconfigured-named-alias
}
