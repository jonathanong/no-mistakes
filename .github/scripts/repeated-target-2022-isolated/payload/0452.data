import { read, sql } from '@example/db';

// An opaque consumer may execute callbacks held in live array slots.
export function arrayCallbacks(consume: unknown) {
  const query = sql`SELECT 1`;
  consume([() => {
    query.append(sql`; DELETE FROM records`);
    read(query);
  }]);
}
