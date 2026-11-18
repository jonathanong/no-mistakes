declare function consume(value: unknown): void;
const moduleReady: unknown[] = [];
export function moduleBuilder(groups: Record<string, number>) {
  Object.entries(groups).forEach(entry => { moduleReady.push(entry); });
}
export function unknownReceiver(rows: unknown) {
  const ready: unknown[] = [];
  (rows as number[]).map(value => { ready.push(value); return value; });
}
export function anyReceiver(rows: any) {
  const ready: unknown[] = [];
  rows.map(value => { ready.push(value); return value; });
}
export function castAlias(rows: unknown) {
  const entries = rows as number[];
  const ready: number[] = [];
  entries.map(value => { ready.push(value); return value; });
}
export function custom(rows: { map(callback: (value: number) => number): unknown }) {
  const ready: number[] = [];
  rows.map(value => { ready.push(value); return value; });
}
export function overridden(groups: Record<string, number>) {
  const entries = Object.entries(groups);
  entries.map = callback => callback(entries[0]);
  const ready: unknown[] = [];
  entries.map(value => { ready.push(value); return value; });
}
export function escaped(groups: Record<string, number>) {
  const entries = Object.entries(groups);
  consume(entries);
  const ready: unknown[] = [];
  entries.forEach(value => { ready.push(value); });
}
export function asynchronous(groups: Record<string, number>) {
  const ready: unknown[] = [];
  Object.values(groups).forEach(async value => { ready.push(value); });
}
export function generator(groups: Record<string, number>) {
  const ready: unknown[] = [];
  Object.keys(groups).map(function* (value) { ready.push(value); yield value; });
}
export function escapingOutput(groups: Record<string, number>) {
  const ready: unknown[] = [];
  Object.entries(groups).forEach(value => { ready.push(value); consume(ready); });
}
export function shadowedObject(Object: { entries(value: unknown): number[] }) {
  const ready: number[] = [];
  Object.entries({}).map(value => { ready.push(value); return value; });
}
export function shadowedArray(Array: { from(value: unknown): number[] }) {
  const ready: number[] = [];
  Array.from([]).map(value => { ready.push(value); return value; });
}
type Custom = number[] & { map(callback: (value: number) => number): unknown };
export function typedOverride(rows: Custom) {
  const ready: number[] = [];
  rows.map(value => { ready.push(value); return value; });
}
export function mapperEscapes(input: number[]) {
  const ready: number[] = [];
  const rows = Array.from(input, value => { consume(ready); return value; });
  rows.forEach(value => { ready.push(value); });
}
export function entriesContents(groups: Record<string, { ready: boolean }>) {
  const rows = Object.entries(groups);
  rows[0][1].ready = true;
}
export function valuesContents(groups: Record<string, { ready: boolean }>) {
  Object.values(groups)[0].ready = true;
}
export function fromContents(input: Array<{ ready: boolean }>) {
  Array.from(input)[0].ready = true;
}
export function expressionEffects(groups: Record<string, number>) {
  const ready: unknown[] = [];
  Object.entries((consume(ready), groups)).map(value => { ready.push(value); return value; });
}
export function nestedReceiverEscape(rows: number[]) {
  consume({ rows });
  const ready: number[] = [];
  rows.forEach(value => { ready.push(value); });
}
declare function getRows(): number[];
export function escapedTypedResult() {
  const rows = getRows();
  consume(rows);
  const ready: number[] = [];
  rows.forEach(value => { ready.push(value); });
}
export function cyclicAliases() {
  const left: number[] = right;
  const right: number[] = left;
  const ready: number[] = [];
  right.map(value => { ready.push(value); return value; });
}
export function reassignedSource(input: unknown) {
  let rows: number[] = [1];
  rows = input as number[];
  const ready: number[] = [];
  rows.forEach(value => { ready.push(value); });
}
export function packedEscape() {
  const rows = getRows();
  consume([rows]);
  const ready: number[] = [];
  rows.forEach(value => { ready.push(value); });
}
