import { query } from '@example/db';
declare function getIt(): any;

// Runtime SQL remains unknown: memberCallee.
getIt().each(['first', 'second'])('memberCallee %s', async field => {
  const map = { first: { sql: 'safe_one', value: 1 }, second: { sql: 'safe_two', value: 2 } };
  const selected = map[field];
  await query(`/* memberCallee */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});
