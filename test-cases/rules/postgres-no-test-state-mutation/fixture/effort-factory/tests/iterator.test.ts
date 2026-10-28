import { it } from 'vitest';
import { beginTransaction } from '../db/index';

declare const unknownId: string;
it.each(['safe', 'bad'] as const)('relative factory with finite SQL', async field => {
  const tx = await beginTransaction();
  await tx('SELECT 1'); // setup before target must stay clear
  const whole = {
    safe: { sql: 'SELECT 2', value: unknownId },
    bad: { sql: 'DROP TABLE app.factory_whole', value: unknownId }, // factory whole DROP origin
  }[field];
  await tx(`${whole.sql}`, [whole.value]); // finite whole SQL at factory
  const fragments = {
    safe: { sql: "name = 'ok'", value: unknownId },
    bad: { sql: "name = 'bad'; DROP TABLE app.factory_fragment", value: unknownId }, // factory fragment DROP origin
  };
  const update = fragments[field];
  await tx(`UPDATE app.jobs SET ${update.sql}`, [update.value]); // finite fragment SQL at factory
});
