import { pool as unrelated } from '@other/db';
import type { pool as erased } from '@example/db';
const { query: wrongModule } = unrelated;
wrongModule('DROP TABLE unrelated_module');
const { query: typeOnly } = erased;
typeOnly('DROP TABLE erased_import');
const local = { query: (_sql: string) => {} };
const { query: localQuery } = local;
localQuery('DROP TABLE local_object');
function shadow(pool: { query(sql: string): void }) {
  const { query } = pool;
  query('DROP TABLE shadowed_pool');
}
shadow(local);
let unrelatedAssigned;
unrelatedAssigned = unrelated.query.bind(unrelated);
unrelatedAssigned('DROP TABLE assigned_unrelated_module');
let localAssigned;
localAssigned = local.query.bind(local);
localAssigned('DROP TABLE assigned_local_object');
