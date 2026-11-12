import { query } from './db.mjs';
declare const ids: string[];
declare const parentId: string;
declare const batchSize: number;
type Stage = { run: (q: typeof query, id: string, n: number) => Promise<unknown> };
const STAGES: readonly Stage[] = [
  { run: (q, id, n) => q('DELETE FROM a WHERE parent_id = $1 LIMIT $2', [id, n]) },
  { run: function(q, id, n) { return q('DELETE FROM b WHERE parent_id = $1 LIMIT $2', [id, n]); } },
];
for (const stage of STAGES) {
  await stage.run(query, parentId, batchSize);
}
async function both(a: () => Promise<unknown>, b: () => Promise<unknown>) { await a(); await b(); }
for (const id of ids) {
  await both(() => query('SELECT a', [id]), function() { return query('SELECT b', [id]); });
}
function ignored(callback: () => Promise<unknown>) {}
const stored: (() => Promise<unknown>)[] = [];
function store(callback: () => Promise<unknown>) { stored.push(callback); }
declare function unresolved(callback: () => Promise<unknown>): void;
for (const id of ids) {
  ignored(() => query('ignored', [id]));
  store(() => query('stored', [id]));
  unresolved(() => query('unresolved', [id]));
}
const objectStage = { run: () => query('object') };
for (const id of ids) {
  await objectStage.run();
}
declare const unknownRun: Stage['run'];
const UNKNOWN_STAGES = [{ run: unknownRun }];
for (const stage of UNKNOWN_STAGES) {
  await stage.run(query, parentId, batchSize);
}
const MIXED_STAGES = [{ run: () => query('known') }, { run: unknownRun }];
for (const stage of MIXED_STAGES) {
  await stage.run();
}
let mutableStages = [{ run: () => query('mutable') }];
for (const stage of mutableStages) {
  await stage.run();
}
declare function escape(value: unknown): void;
const ESCAPED_STAGES = [{ run: () => query('escaped') }];
escape(ESCAPED_STAGES);
for (const stage of ESCAPED_STAGES) {
  await stage.run();
}
const UNUSED_STAGES = [{ run: () => query('unused') }];
for (const stage of UNUSED_STAGES) {
  ignored(stage.run);
}
const SELF_STAGES = [{ run: (value: unknown) => query('self') }];
for (const stage of SELF_STAGES) {
  await stage.run(stage);
}
const DIRECT_STAGES = [{ run: () => query('direct') }];
for (const stage of DIRECT_STAGES) {
  await stage.run();
}
const RECEIVER_STAGES = [{ run: function() { this.run = async () => {}; return query('receiver'); } }];
for (const stage of RECEIVER_STAGES) {
  await stage.run();
  await stage.run();
}
for (const stage of [{ run: () => query('inline') }]) {
  await stage.run();
}
const MUTATED_STAGES = [{ run: () => query('mutated') }];
MUTATED_STAGES[0].run = async () => {};
for (const stage of MUTATED_STAGES) {
  await stage.run();
}
