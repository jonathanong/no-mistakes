import { read, sql } from '@example/db';

function branchStatement(flag: boolean) {
  if (flag) return '/* branch/read */ SELECT id FROM records';
  return '/* branch/read */ SELECT name FROM records';
}
export function branches(flag: boolean) { read(branchStatement(flag)); }

function mappedCtes() {
  const parameters = [];
  const partitions = [{ name: 'first_input', type: 'text' }, { name: 'second_input', type: 'uuid' }];
  const ctes = partitions.map(partition => {
    const index = parameters.length + 1;
    parameters.push('value');
    return `${partition.name} AS (SELECT unnest($${index}::${partition.type}[]) AS value)`;
  });
  return `/* collections/map */ WITH ${ctes.join(', ')} SELECT value FROM first_input`;
}
export function mapping() { read(mappedCtes()); }

function conditionalCtes(flag: boolean) {
  const ctes = [];
  if (flag) ctes.push('selected AS (SELECT id FROM records)');
  if (ctes.length === 0) return '/* collections/conditional */ SELECT id FROM records';
  return `/* collections/conditional */ WITH ${ctes.join(', ')} SELECT id FROM selected`;
}
export function conditional(flag: boolean) { read(conditionalCtes(flag)); }

function appendCtes(query: unknown, ctes: unknown[]) {
  if (ctes.length === 0) return;
  query.append(sql`WITH `);
  ctes.forEach((cte, index) => {
    if (index > 0) query.append(sql`, `);
    query.append(cte);
  });
}
export function appending() {
  const query = sql`/* collections/append */ `;
  appendCtes(query, [sql`first AS (SELECT 1)`, sql`second AS (SELECT 2)`]);
  query.append(sql` SELECT * FROM first`);
  read(query);
}

// A WITH prefix says nothing about writes hidden in one recovered alternative.
function mixed(flag: boolean) {
  if (flag) return '/* branch/mixed */ WITH removed AS (DELETE FROM records RETURNING id) SELECT id FROM removed';
  return '/* branch/mixed */ WITH selected AS (SELECT id FROM records) SELECT id FROM selected';
}
export function mixedEffects(flag: boolean) { read(mixed(flag)); }

export function unknownCollection(partitions: unknown[]) {
  read(`/* collections/runtime */ WITH ${partitions.map(partition => 'chosen AS (SELECT 1)').join(', ')} SELECT * FROM chosen`);
}
export function unknownCallback(callback: unknown) {
  read(`/* collections/callback */ WITH ${['chosen'].map(callback).join(', ')} SELECT 1`);
}
export function unknownSpread(input: unknown[]) {
  const parts = [...input];
  read(`/* collections/spread */ WITH ${parts.join(', ')} SELECT 1`);
}
export function getter(input: unknown) {
  read(`/* collections/getter */ SELECT ${input.fragment}`);
}
export function emptyAppending() {
  const query = sql`/* collections/empty-append */ SELECT 1`;
  appendCtes(query, []);
  read(query);
}
