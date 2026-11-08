import { query } from '@example/db';
import { candidates } from './templates';

export async function loadCandidates() {
  await query(candidates());
}
