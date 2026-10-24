import { read } from '@example/db';
import { other } from '@other/db';
import type { QueryOptions } from '@example/db';
export function mixed(options: QueryOptions) {
  other('SELECT 1', options); // ADVISE: this profile does not own other().
  read('SELECT 1', options); // CLEAR: the matching profile owns this metadata.
}
