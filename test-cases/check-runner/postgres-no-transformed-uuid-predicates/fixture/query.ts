import { query } from '@db';

export function findUsersAfter(timestamp: string) {
  return query(
    'SELECT id FROM users WHERE uuid_extract_timestamp(id) > $1',
    [timestamp],
  );
}
