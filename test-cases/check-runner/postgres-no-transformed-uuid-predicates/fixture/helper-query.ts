import { query } from '@db';

function buildUsersAfter() {
  return 'SELECT id FROM users WHERE uuid_extract_timestamp(id) > $1';
}

export function findUsersAfter(timestamp: string) {
  return query(buildUsersAfter(), [timestamp]);
}
